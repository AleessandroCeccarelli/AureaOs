// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Bounded Vulkan texture cache for compositor-owned desktop artwork.

use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt,
    sync::Arc,
};

use crate::{desktop::DesktopIconTextureKey, render::shm_buffer::ShmBufferSnapshot};

use super::{VulkanLogicalDevice, VulkanPendingUpload, VulkanShmTexture, VulkanShmTextureError};

const MAX_DESKTOP_ICON_TEXTURES: usize = 32;
const MAX_DESKTOP_ICON_BYTES: usize = 16 * 1024 * 1024;
const MAX_PENDING_ICON_BATCHES: usize = 2;
const MAX_PENDING_ICON_BYTES: usize = 8 * 1024 * 1024;

/// Errors returned while preparing native desktop icon textures.
#[derive(Debug)]
pub(crate) enum VulkanDesktopIconTextureCacheError {
    Texture(VulkanShmTextureError),
    CacheBudgetExceeded,
    InvalidAssetDimensions,
}

impl fmt::Display for VulkanDesktopIconTextureCacheError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Texture(error) => {
                write!(formatter, "Desktop icon texture upload failed: {error}")
            }
            Self::CacheBudgetExceeded => {
                formatter.write_str("Desktop icon textures exceed the cache resource budget")
            }
            Self::InvalidAssetDimensions => {
                formatter.write_str("Desktop icon asset dimensions do not match its cache key")
            }
        }
    }
}

impl Error for VulkanDesktopIconTextureCacheError {}

struct CachedDesktopIconTexture {
    texture: Arc<VulkanShmTexture>,
    byte_len: usize,
}

struct PendingDesktopIconBatch {
    upload: VulkanPendingUpload<Vec<Arc<VulkanShmTexture>>>,
    keys: Vec<DesktopIconTextureKey>,
    byte_len: usize,
}

/// Owns native desktop artwork independently from client SHM and wallpaper caches.
pub(crate) struct VulkanDesktopIconTextureCache {
    device: Arc<VulkanLogicalDevice>,
    entries: HashMap<DesktopIconTextureKey, CachedDesktopIconTexture>,
    pending_uploads: Vec<PendingDesktopIconBatch>,
    cached_bytes: usize,
    pending_bytes: usize,
}

impl VulkanDesktopIconTextureCache {
    pub(crate) fn new(device: Arc<VulkanLogicalDevice>) -> Self {
        Self {
            device,
            entries: HashMap::new(),
            pending_uploads: Vec::new(),
            cached_bytes: 0,
            pending_bytes: 0,
        }
    }

    /// Starts missing uploads and keeps their GPU resources alive until fences signal.
    pub(crate) fn prepare(
        &mut self,
        requested: &[(DesktopIconTextureKey, ShmBufferSnapshot)],
    ) -> Result<(), VulkanDesktopIconTextureCacheError> {
        if self.device.is_unusable() {
            return Err(VulkanDesktopIconTextureCacheError::Texture(VulkanShmTextureError::Device(vk::Result::ERROR_DEVICE_LOST)));
        }
        self.reap_completed_uploads()?;
        let requested_keys = requested
            .iter()
            .map(|(key, _)| *key)
            .collect::<HashSet<_>>();
        let pending_keys = self
            .pending_uploads
            .iter()
            .flat_map(|batch| batch.keys.iter().copied())
            .collect::<HashSet<_>>();
        self.entries
            .retain(|key, _| requested_keys.contains(key) || pending_keys.contains(key));
        self.recalculate_cached_bytes();

        if requested_keys.len() > MAX_DESKTOP_ICON_TEXTURES {
            return Err(VulkanDesktopIconTextureCacheError::CacheBudgetExceeded);
        }

        let mut missing = Vec::new();
        let mut missing_bytes = 0usize;
        let mut seen = HashSet::new();
        for (key, snapshot) in requested {
            if snapshot.width() != key.pixel_size
                || snapshot.height() != key.pixel_size
                || key.pixel_size == 0
                || key.pixel_size > 256
            {
                return Err(VulkanDesktopIconTextureCacheError::InvalidAssetDimensions);
            }
            if !seen.insert(*key) {
                continue;
            }
            if self.entries.contains_key(key) {
                continue;
            }

            let byte_len = snapshot.pixels().len();
            missing_bytes = missing_bytes
                .checked_add(byte_len)
                .ok_or(VulkanDesktopIconTextureCacheError::CacheBudgetExceeded)?;
            missing.push((*key, snapshot.clone(), byte_len));
        }

        if self
            .cached_bytes
            .checked_add(missing_bytes)
            .map_or(true, |total| total > MAX_DESKTOP_ICON_BYTES)
        {
            return Err(VulkanDesktopIconTextureCacheError::CacheBudgetExceeded);
        }

        if missing.is_empty() {
            return Ok(());
        }

        self.apply_upload_backpressure(missing_bytes)?;

        let snapshots = missing
            .iter()
            .map(|(_, snapshot, _)| snapshot.clone())
            .collect();
        let upload = VulkanShmTexture::submit_snapshots(Arc::clone(&self.device), snapshots)
            .map_err(VulkanDesktopIconTextureCacheError::Texture)?;

        let keys = missing.iter().map(|(key, _, _)| *key).collect::<Vec<_>>();
        for ((key, _, byte_len), texture) in
            missing.into_iter().zip(upload.resources().iter().cloned())
        {
            self.cached_bytes = self.cached_bytes.saturating_add(byte_len);
            self.entries
                .insert(key, CachedDesktopIconTexture { texture, byte_len });
        }

        self.pending_bytes = self.pending_bytes.saturating_add(missing_bytes);
        self.pending_uploads.push(PendingDesktopIconBatch {
            upload,
            keys,
            byte_len: missing_bytes,
        });
        Ok(())
    }

    /// Polls completed upload fences without blocking the compositor frame.
    pub(crate) fn poll(&mut self) -> Result<(), VulkanDesktopIconTextureCacheError> {
        self.reap_completed_uploads()
    }

    /// Completes all uploads before the renderer borrows cached texture views.
    pub(crate) fn complete_pending_uploads(
        &mut self,
    ) -> Result<(), VulkanDesktopIconTextureCacheError> {
        while !self.pending_uploads.is_empty() {
            self.complete_oldest_upload()?;
        }
        Ok(())
    }

    pub(crate) fn texture(&self, key: DesktopIconTextureKey) -> Option<&VulkanShmTexture> {
        self.entries.get(&key).map(|entry| entry.texture.as_ref())
    }

    fn reap_completed_uploads(&mut self) -> Result<(), VulkanDesktopIconTextureCacheError> {
        let mut index = 0;
        while index < self.pending_uploads.len() {
            let complete = match self.pending_uploads[index].upload.is_complete() {
                Ok(complete) => complete,
                Err(error) => {
                    let batch = self.pending_uploads.remove(index);
                    self.pending_bytes = self.pending_bytes.saturating_sub(batch.byte_len);
                    for key in batch.keys {
                        self.remove_key(key);
                    }
                    return Err(VulkanDesktopIconTextureCacheError::Texture(
                        VulkanShmTextureError::Command(error),
                    ));
                }
            };
            if !complete {
                index += 1;
                continue;
            }

            let batch = self.pending_uploads.remove(index);
            self.pending_bytes = self.pending_bytes.saturating_sub(batch.byte_len);
            if let Err(error) = batch.upload.wait() {
                for key in batch.keys {
                    self.remove_key(key);
                }
                return Err(VulkanDesktopIconTextureCacheError::Texture(
                    VulkanShmTextureError::Command(error),
                ));
            }
        }
        Ok(())
    }

    fn apply_upload_backpressure(
        &mut self,
        incoming_bytes: usize,
    ) -> Result<(), VulkanDesktopIconTextureCacheError> {
        while self.pending_uploads.len() >= MAX_PENDING_ICON_BATCHES
            || self
                .pending_bytes
                .checked_add(incoming_bytes)
                .map_or(true, |total| total > MAX_PENDING_ICON_BYTES)
        {
            if self.pending_uploads.is_empty() {
                return Err(VulkanDesktopIconTextureCacheError::CacheBudgetExceeded);
            }
            self.complete_oldest_upload()?;
        }
        Ok(())
    }

    fn complete_oldest_upload(&mut self) -> Result<(), VulkanDesktopIconTextureCacheError> {
        let batch = self.pending_uploads.remove(0);
        self.pending_bytes = self.pending_bytes.saturating_sub(batch.byte_len);
        if let Err(error) = batch.upload.wait() {
            for key in batch.keys {
                self.remove_key(key);
            }
            return Err(VulkanDesktopIconTextureCacheError::Texture(
                VulkanShmTextureError::Command(error),
            ));
        }
        Ok(())
    }

    fn remove_key(&mut self, key: DesktopIconTextureKey) {
        if let Some(entry) = self.entries.remove(&key) {
            self.cached_bytes = self.cached_bytes.saturating_sub(entry.byte_len);
        }
    }

    fn recalculate_cached_bytes(&mut self) {
        self.cached_bytes = self.entries.values().map(|entry| entry.byte_len).sum();
    }
}
