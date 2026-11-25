// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! One-time GPU initialization for the compositor-owned native cursor.

use std::{error::Error, fmt, sync::Arc};

use super::{VulkanLogicalDevice, VulkanShmTexture, VulkanShmTextureError};

/// Errors produced while creating or uploading the native cursor texture.
#[derive(Debug)]
pub(crate) enum VulkanNativeCursorTextureError {
    Snapshot(String),
    EmptyUpload,
    Texture(VulkanShmTextureError),
}

impl fmt::Display for VulkanNativeCursorTextureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Snapshot(error) => write!(formatter, "Native cursor snapshot: {error}"),
            Self::EmptyUpload => formatter.write_str("Native cursor upload completed without a texture"),
            Self::Texture(error) => write!(formatter, "Native cursor texture: {error}"),
        }
    }
}

impl Error for VulkanNativeCursorTextureError {}

/// Owns the built-in arrow texture for the lifetime of the Vulkan device.
///
/// The arrow is only 32×32 RGBA pixels. Its one-time upload is completed before
/// the first frame that needs it, so the initial desktop frame cannot silently
/// omit the pointer while waiting for a later input event to trigger a redraw.
pub(crate) struct VulkanNativeCursorTexture {
    texture: Option<Arc<VulkanShmTexture>>,
}

impl VulkanNativeCursorTexture {
    pub(crate) fn new() -> Self {
        Self { texture: None }
    }

    /// Initializes the arrow texture once and returns whether it is ready.
    pub(crate) fn ensure(
        &mut self,
        device: Arc<VulkanLogicalDevice>,
    ) -> Result<bool, VulkanNativeCursorTextureError> {
        if self.texture.is_some() {
            return Ok(true);
        }

        let snapshot = crate::render::cursor::default_cursor_snapshot()
            .map_err(|error| VulkanNativeCursorTextureError::Snapshot(error.to_string()))?;
        let upload = VulkanShmTexture::submit_snapshots(device, vec![snapshot])
            .map_err(VulkanNativeCursorTextureError::Texture)?;
        let mut textures = upload.wait().map_err(|error| {
            VulkanNativeCursorTextureError::Texture(VulkanShmTextureError::Command(error))
        })?;
        let texture = textures
            .pop()
            .ok_or(VulkanNativeCursorTextureError::EmptyUpload)?;
        self.texture = Some(texture);
        Ok(true)
    }

    pub(crate) fn texture(&self) -> Option<&VulkanShmTexture> {
        self.texture.as_deref()
    }
}
