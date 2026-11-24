// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Asynchronous lifetime management for the compositor-owned native cursor.

use std::{error::Error, fmt, sync::Arc};

use super::{
    VulkanLogicalDevice, VulkanPendingUpload, VulkanShmTexture, VulkanShmTextureError,
};

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

/// Owns the native cursor texture and its in-flight upload until its fence signals.
pub(crate) struct VulkanNativeCursorTexture {
    texture: Option<Arc<VulkanShmTexture>>,
    pending_upload: Option<VulkanPendingUpload<Vec<Arc<VulkanShmTexture>>>>,
}

impl VulkanNativeCursorTexture {
    pub(crate) fn new() -> Self {
        Self {
            texture: None,
            pending_upload: None,
        }
    }

    /// Starts an upload or promotes its completed result without blocking a frame.
    pub(crate) fn ensure(
        &mut self,
        device: Arc<VulkanLogicalDevice>,
    ) -> Result<bool, VulkanNativeCursorTextureError> {
        if self.texture.is_some() {
            return Ok(true);
        }

        if let Some(upload) = self.pending_upload.as_ref() {
            let complete = upload
                .is_complete()
                .map_err(|error| {
                    VulkanNativeCursorTextureError::Texture(
                        VulkanShmTextureError::Command(error),
                    )
                })?;
            if !complete {
                return Ok(false);
            }

            let upload = self
                .pending_upload
                .take()
                .expect("pending cursor upload was checked above");
            let mut textures = upload.wait().map_err(|error| {
                VulkanNativeCursorTextureError::Texture(
                    VulkanShmTextureError::Command(error),
                )
            })?;
            let texture = textures
                .pop()
                .ok_or(VulkanNativeCursorTextureError::EmptyUpload)?;
            self.texture = Some(texture);
            return Ok(true);
        }

        let snapshot = crate::render::cursor::default_cursor_snapshot()
            .map_err(|error| VulkanNativeCursorTextureError::Snapshot(error.to_string()))?;
        let upload = VulkanShmTexture::submit_snapshots(device, vec![snapshot])
            .map_err(VulkanNativeCursorTextureError::Texture)?;
        self.pending_upload = Some(upload);
        Ok(false)
    }

    pub(crate) fn texture(&self) -> Option<&VulkanShmTexture> {
        self.texture.as_deref()
    }
}
