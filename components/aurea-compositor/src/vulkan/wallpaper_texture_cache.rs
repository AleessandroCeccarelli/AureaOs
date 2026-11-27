// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Bounded cache for the currently selected desktop wallpaper texture.

use std::{error::Error, fmt, sync::Arc};

use crate::desktop::WallpaperPixels;

use super::{
    VulkanLogicalDevice, VulkanPendingUpload, VulkanWallpaperTexture, VulkanWallpaperTextureError,
};

const MAX_PENDING_WALLPAPER_UPLOADS: usize = 2;
const MAX_PENDING_WALLPAPER_BYTES: usize = 256 * 1024 * 1024;

/// Identifies one selected wallpaper revision independently from Wayland surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WallpaperTextureKey {
    pub(crate) asset_id: String,
    pub(crate) revision: u64,
}

struct PendingWallpaperUpload {
    key: WallpaperTextureKey,
    byte_len: usize,
    upload: VulkanPendingUpload<Arc<VulkanWallpaperTexture>>,
}

/// Errors returned while preparing the selected wallpaper texture.
#[derive(Debug)]
pub(crate) enum VulkanWallpaperTextureCacheError {
    Upload(VulkanWallpaperTextureError),
    Decode(String),
    Command(smithay::reexports::ash::vk::Result),
    PendingBudgetExceeded,
}

impl fmt::Display for VulkanWallpaperTextureCacheError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Upload(error) => write!(formatter, "Wallpaper texture cache upload: {error}"),
            Self::Decode(error) => write!(formatter, "Wallpaper decode service: {error}"),
            Self::Command(error) => {
                write!(formatter, "Wallpaper upload synchronization: {error:?}")
            }
            Self::PendingBudgetExceeded => {
                formatter.write_str("Wallpaper upload queue budget exceeded")
            }
        }
    }
}

impl Error for VulkanWallpaperTextureCacheError {}

/// Owns one ready texture and a small bounded set of fence-backed uploads.
pub(crate) struct VulkanWallpaperTextureCache {
    device: Arc<VulkanLogicalDevice>,
    selected: Option<WallpaperTextureKey>,
    ready: Option<(WallpaperTextureKey, Arc<VulkanWallpaperTexture>)>,
    pending: Vec<PendingWallpaperUpload>,
    pending_bytes: usize,
    failed_upload: Option<WallpaperTextureKey>,
    synchronization_error: Option<smithay::reexports::ash::vk::Result>,
}

impl VulkanWallpaperTextureCache {
    pub(crate) fn new(device: Arc<VulkanLogicalDevice>) -> Self {
        Self {
            device,
            selected: None,
            ready: None,
            pending: Vec::new(),
            pending_bytes: 0,
            failed_upload: None,
            synchronization_error: None,
        }
    }

    /// Reports whether another upload of this size fits the bounded pending budget.
    pub(crate) fn can_accept_upload(&self, byte_len: usize) -> bool {
        self.pending.len() < MAX_PENDING_WALLPAPER_UPLOADS
            && self
                .pending_bytes
                .checked_add(byte_len)
                .is_some_and(|total| total <= MAX_PENDING_WALLPAPER_BYTES)
    }

    /// Selects a new pixel revision and submits its upload without waiting.
    pub(crate) fn prepare(
        &mut self,
        key: WallpaperTextureKey,
        pixels: &WallpaperPixels,
    ) -> Result<bool, VulkanWallpaperTextureCacheError> {
        if self.device.is_unusable() {
            return Err(VulkanWallpaperTextureCacheError::Command(smithay::reexports::ash::vk::Result::ERROR_DEVICE_LOST));
        }
        self.poll()?;
        if self
            .ready
            .as_ref()
            .is_some_and(|(ready_key, _)| ready_key == &key)
        {
            self.selected = Some(key.clone());
            return Ok(true);
        }
        if self.pending.iter().any(|pending| pending.key == key) {
            self.selected = Some(key.clone());
            self.ready = None;
            return Ok(true);
        }

        // Invalidate the previously visible revision before attempting any
        // fallible allocation or submission for the new selection. Pending
        // uploads remain owned until their fences prove completion.
        if self.selected.as_ref() != Some(&key) {
            self.selected = Some(key.clone());
            self.ready = None;
        }

        if self.failed_upload.as_ref() == Some(&key) {
            return Ok(false);
        }
        if self
            .failed_upload
            .as_ref()
            .is_some_and(|failed| failed != &key)
        {
            self.failed_upload = None;
        }

        let byte_len = pixels.byte_len();
        if self.pending.len() >= MAX_PENDING_WALLPAPER_UPLOADS
            || self
                .pending_bytes
                .checked_add(byte_len)
                .map_or(true, |total| total > MAX_PENDING_WALLPAPER_BYTES)
        {
            return Err(VulkanWallpaperTextureCacheError::PendingBudgetExceeded);
        }

        let upload = match VulkanWallpaperTexture::submit_pixels(Arc::clone(&self.device), pixels) {
            Ok(upload) => upload,
            Err(VulkanWallpaperTextureError::Command(
                smithay::reexports::ash::smithay::reexports::ash::vk::Result::ERROR_DEVICE_LOST,
            )) => {
                let error = smithay::reexports::ash::smithay::reexports::ash::vk::Result::ERROR_DEVICE_LOST;
                self.synchronization_error = Some(error);
                return Err(VulkanWallpaperTextureCacheError::Command(error));
            }
            Err(error) => {
                self.failed_upload = Some(key.clone());
                return Err(VulkanWallpaperTextureCacheError::Upload(error));
            }
        };

        self.selected = Some(key.clone());
        self.ready = None;
        self.pending_bytes = self.pending_bytes.saturating_add(byte_len);
        self.pending.push(PendingWallpaperUpload {
            key: key.clone(),
            byte_len,
            upload,
        });
        Ok(true)
    }

    /// Returns whether a decode or GPU upload still needs another render-frame poll.
    pub(crate) fn has_pending_work(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Polls upload fences without blocking; only completed selected textures become visible.
    pub(crate) fn poll(&mut self) -> Result<(), VulkanWallpaperTextureCacheError> {
        if let Some(error) = self.synchronization_error {
            return Err(VulkanWallpaperTextureCacheError::Command(error));
        }

        let mut index = 0;
        while index < self.pending.len() {
            let complete = match self.pending[index].upload.is_complete() {
                Ok(complete) => complete,
                Err(error) => {
                    // A fence error makes this cache unsafe to advance. Retain
                    // pending ownership and stop polling instead of repeatedly
                    // querying a failed device synchronization primitive.
                    self.synchronization_error = Some(error);
                    return Err(VulkanWallpaperTextureCacheError::Command(error));
                }
            };
            if !complete {
                index += 1;
                continue;
            }

            let pending = self.pending.remove(index);
            self.pending_bytes = self.pending_bytes.saturating_sub(pending.byte_len);
            let texture = match pending.upload.wait() {
                Ok(texture) => texture,
                Err(error) => {
                    self.synchronization_error = Some(error);
                    return Err(VulkanWallpaperTextureCacheError::Command(error));
                }
            };
            if self.selected.as_ref() == Some(&pending.key) {
                self.ready = Some((pending.key, texture));
            }
        }
        Ok(())
    }

    pub(crate) fn texture(&self) -> Option<&VulkanWallpaperTexture> {
        let (key, texture) = self.ready.as_ref()?;
        (self.selected.as_ref() == Some(key)).then_some(texture.as_ref())
    }

    /// Returns the ready texture only if the selected cache key belongs to this asset.
    pub(crate) fn texture_for_asset(&self, asset_id: &str) -> Option<&VulkanWallpaperTexture> {
        let (key, texture) = self.ready.as_ref()?;
        (self.selected.as_ref() == Some(key) && key.asset_id == asset_id)
            .then_some(texture.as_ref())
    }

    pub(crate) fn clear(&mut self) {
        self.selected = None;
        self.ready = None;
        self.failed_upload = None;
    }
}
