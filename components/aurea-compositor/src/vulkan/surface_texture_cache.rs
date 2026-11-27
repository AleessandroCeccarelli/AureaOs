// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Bounded, per-device cache of uploaded Wayland SHM surface textures.

use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt,
    sync::Arc,
};

use smithay::reexports::wayland_server::backend::ObjectId;

use crate::render::shm_buffer::ShmBufferSnapshot;

use super::{VulkanLogicalDevice, VulkanPendingUpload, VulkanShmTexture, VulkanShmTextureError};

const MAX_CACHED_SURFACE_TEXTURES: usize = 128;
const MAX_CACHED_SURFACE_TEXTURE_BYTES: usize = 256 * 1024 * 1024;
const MAX_PENDING_UPLOAD_BATCHES: usize = 2;
const MAX_PENDING_UPLOAD_BYTES: usize = 128 * 1024 * 1024;

/// Errors returned while synchronizing the SHM texture cache.
#[derive(Debug)]
pub(crate) enum VulkanSurfaceTextureCacheError {
    Texture(VulkanShmTextureError),
    CacheBudgetExceeded,
}

impl fmt::Display for VulkanSurfaceTextureCacheError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Texture(error) => write!(formatter, "SHM texture cache upload failed: {error}"),
            Self::CacheBudgetExceeded => {
                formatter.write_str("Visible SHM textures exceed the cache resource budget")
            }
        }
    }
}

impl Error for VulkanSurfaceTextureCacheError {}

struct CachedSurfaceTexture {
    revision: u64,
    snapshot: Arc<ShmBufferSnapshot>,
    texture: Arc<VulkanShmTexture>,
    byte_len: usize,
    last_used: u64,
}

struct PendingSurfaceTextureBatch {
    upload: VulkanPendingUpload<Vec<Arc<VulkanShmTexture>>>,
    surface_ids: Vec<ObjectId>,
    byte_len: usize,
}

/// Retains uploaded textures while their Wayland surfaces remain active.
pub(crate) struct VulkanSurfaceTextureCache {
    device: Arc<VulkanLogicalDevice>,
    entries: HashMap<ObjectId, CachedSurfaceTexture>,
    pending_uploads: Vec<PendingSurfaceTextureBatch>,
    cached_bytes: usize,
    pending_bytes: usize,
    frame: u64,
}

impl VulkanSurfaceTextureCache {
    pub(crate) fn new(device: Arc<VulkanLogicalDevice>) -> Self {
        Self {
            device,
            entries: HashMap::new(),
            pending_uploads: Vec::new(),
            cached_bytes: 0,
            pending_bytes: 0,
            frame: 0,
        }
    }

    /// Starts missing texture uploads and retains their resources until their fences signal.
    pub(crate) fn prepare(
        &mut self,
        active_surface_ids: &[ObjectId],
        visible_surfaces: &[(ObjectId, u64, Arc<ShmBufferSnapshot>)],
    ) -> Result<(), VulkanSurfaceTextureCacheError> {
        if self.device.is_unusable() {
            return Err(VulkanSurfaceTextureCacheError::Texture(VulkanShmTextureError::Command(smithay::reexports::ash::vk::Result::ERROR_DEVICE_LOST)));
        }
        self.reap_completed_uploads()?;
        self.frame = self.frame.wrapping_add(1);
        let active: HashSet<_> = active_surface_ids.iter().cloned().collect();
        self.entries.retain(|id, _| active.contains(id));
        self.recalculate_cached_bytes();

        let required: HashSet<_> = visible_surfaces
            .iter()
            .map(|(id, _, _)| id.clone())
            .collect();
        let requested_bytes = visible_surfaces
            .iter()
            .try_fold(0usize, |total, (_, _, snapshot)| {
                total.checked_add(snapshot.pixels().len())
            })
            .ok_or(VulkanSurfaceTextureCacheError::CacheBudgetExceeded)?;

        if visible_surfaces.len() > MAX_CACHED_SURFACE_TEXTURES
            || requested_bytes > MAX_CACHED_SURFACE_TEXTURE_BYTES
        {
            return Err(VulkanSurfaceTextureCacheError::CacheBudgetExceeded);
        }

        for (surface_id, revision, snapshot) in visible_surfaces {
            let stale = self.entries.get(surface_id).is_some_and(|entry| {
                entry.revision != *revision || !Arc::ptr_eq(&entry.snapshot, snapshot)
            });
            if stale {
                self.remove_surface(surface_id);
            }
        }

        let mut missing = Vec::new();
        let mut missing_bytes = 0usize;
        for (surface_id, revision, snapshot) in visible_surfaces {
            if let Some(entry) = self.entries.get_mut(surface_id)
                && entry.revision == *revision
                && Arc::ptr_eq(&entry.snapshot, snapshot)
            {
                entry.last_used = self.frame;
                continue;
            }

            let byte_len = snapshot.pixels().len();
            missing_bytes = missing_bytes
                .checked_add(byte_len)
                .ok_or(VulkanSurfaceTextureCacheError::CacheBudgetExceeded)?;
            missing.push((
                surface_id.clone(),
                *revision,
                Arc::clone(snapshot),
                byte_len,
            ));
        }

        while self.entries.len().saturating_add(missing.len()) > MAX_CACHED_SURFACE_TEXTURES
            || self
                .cached_bytes
                .checked_add(missing_bytes)
                .map_or(true, |total| total > MAX_CACHED_SURFACE_TEXTURE_BYTES)
        {
            let candidate = self
                .entries
                .iter()
                .filter(|(id, _)| !required.contains(*id))
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(id, _)| id.clone());
            let Some(candidate) = candidate else {
                return Err(VulkanSurfaceTextureCacheError::CacheBudgetExceeded);
            };
            self.remove_surface(&candidate);
        }

        if missing.is_empty() {
            return Ok(());
        }

        self.apply_upload_backpressure(missing_bytes)?;

        let snapshots = missing
            .iter()
            .map(|(_, _, snapshot, _)| snapshot.as_ref().clone())
            .collect();
        let upload = VulkanShmTexture::submit_snapshots(Arc::clone(&self.device), snapshots)
            .map_err(VulkanSurfaceTextureCacheError::Texture)?;

        let surface_ids = missing
            .iter()
            .map(|(surface_id, _, _, _)| surface_id.clone())
            .collect::<Vec<_>>();

        for ((surface_id, revision, snapshot, byte_len), texture) in
            missing.into_iter().zip(upload.resources().iter().cloned())
        {
            self.cached_bytes = self.cached_bytes.saturating_add(byte_len);
            self.entries.insert(
                surface_id,
                CachedSurfaceTexture {
                    revision,
                    snapshot,
                    texture,
                    byte_len,
                    last_used: self.frame,
                },
            );
        }

        self.pending_bytes = self.pending_bytes.saturating_add(missing_bytes);
        self.pending_uploads.push(PendingSurfaceTextureBatch {
            upload,
            surface_ids,
            byte_len: missing_bytes,
        });

        Ok(())
    }

    /// Polls completed uploads without blocking the compositor thread.
    fn reap_completed_uploads(&mut self) -> Result<(), VulkanSurfaceTextureCacheError> {
        let mut index = 0;
        while index < self.pending_uploads.len() {
            let complete = match self.pending_uploads[index].upload.is_complete() {
                Ok(complete) => complete,
                Err(error) => {
                    let batch = self.pending_uploads.remove(index);
                    self.pending_bytes = self.pending_bytes.saturating_sub(batch.byte_len);
                    for surface_id in batch.surface_ids {
                        self.remove_surface(&surface_id);
                    }
                    return Err(VulkanSurfaceTextureCacheError::Texture(
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
                for surface_id in batch.surface_ids {
                    self.remove_surface(&surface_id);
                }
                return Err(VulkanSurfaceTextureCacheError::Texture(
                    VulkanShmTextureError::Command(error),
                ));
            }
        }
        Ok(())
    }

    /// Applies bounded backpressure before another staging allocation is submitted.
    fn apply_upload_backpressure(
        &mut self,
        incoming_bytes: usize,
    ) -> Result<(), VulkanSurfaceTextureCacheError> {
        while self.pending_uploads.len() >= MAX_PENDING_UPLOAD_BATCHES
            || self
                .pending_bytes
                .checked_add(incoming_bytes)
                .map_or(true, |total| total > MAX_PENDING_UPLOAD_BYTES)
        {
            if self.pending_uploads.is_empty() {
                return Err(VulkanSurfaceTextureCacheError::CacheBudgetExceeded);
            }
            self.complete_oldest_upload()?;
        }
        Ok(())
    }

    /// Waits for the oldest batch only when the in-flight budget is exhausted.
    fn complete_oldest_upload(&mut self) -> Result<(), VulkanSurfaceTextureCacheError> {
        let batch = self.pending_uploads.remove(0);
        self.pending_bytes = self.pending_bytes.saturating_sub(batch.byte_len);
        if let Err(error) = batch.upload.wait() {
            for surface_id in batch.surface_ids {
                self.remove_surface(&surface_id);
            }
            return Err(VulkanSurfaceTextureCacheError::Texture(
                VulkanShmTextureError::Command(error),
            ));
        }
        Ok(())
    }

    /// Completes every upload before the renderer samples cached textures.
    pub(crate) fn complete_pending_uploads(
        &mut self,
    ) -> Result<(), VulkanSurfaceTextureCacheError> {
        while !self.pending_uploads.is_empty() {
            self.complete_oldest_upload()?;
        }
        Ok(())
    }

    pub(crate) fn texture(&self, surface_id: &ObjectId) -> Option<&VulkanShmTexture> {
        self.entries
            .get(surface_id)
            .map(|entry| entry.texture.as_ref())
    }

    fn remove_surface(&mut self, surface_id: &ObjectId) {
        if let Some(entry) = self.entries.remove(surface_id) {
            self.cached_bytes = self.cached_bytes.saturating_sub(entry.byte_len);
        }
    }

    fn recalculate_cached_bytes(&mut self) {
        self.cached_bytes = self.entries.values().map(|entry| entry.byte_len).sum();
    }
}
