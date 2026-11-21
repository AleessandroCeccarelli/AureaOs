// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Registers CPU-upload GBM allocations as explicit-modifier KMS framebuffers.

use std::{error::Error, fmt};

use smithay::{
    backend::{allocator::{Format, Modifier}, drm::DrmDeviceFd},
    reexports::drm::{
        buffer::PlanarBuffer,
        control::{Device, FbCmd2Flags, framebuffer},
    },
};

use super::{gbm_buffer_writer::GbmCpuUploadBuffer, presentation::DrmFramebufferId};

/// Failure while registering a GBM allocation with the DRM device.
#[derive(Debug)]
pub(crate) struct GbmFramebufferRegistrationError(String);

impl fmt::Display for GbmFramebufferRegistrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "GBM framebuffer registration failed: {}", self.0)
    }
}

impl Error for GbmFramebufferRegistrationError {}

/// Owns a KMS framebuffer handle and the DRM descriptor needed to destroy it.
pub(crate) struct StrictGbmFramebuffer {
    handle: framebuffer::Handle,
    format: Format,
    drm: DrmDeviceFd,
}

impl StrictGbmFramebuffer {
    /// Registers a GBM buffer using AddFB2 only; no legacy fallback is attempted.
    pub(crate) fn register(
        drm: &DrmDeviceFd,
        buffer: &GbmCpuUploadBuffer,
    ) -> Result<Self, GbmFramebufferRegistrationError> {
        let gbm = buffer.gbm_buffer();
        let modifier = PlanarBuffer::modifier(gbm);
        let flags = if modifier.is_some() {
            FbCmd2Flags::MODIFIERS
        } else {
            FbCmd2Flags::empty()
        };

        let handle = drm
            .add_planar_framebuffer(gbm, flags)
            .map_err(|error| GbmFramebufferRegistrationError(error.to_string()))?;

        Ok(Self {
            handle,
            format: Format {
                code: gbm.format(),
                modifier: modifier.unwrap_or(Modifier::Invalid),
            },
            drm: drm.clone(),
        })
    }

    /// Returns the KMS framebuffer handle for plane configuration.
    pub(crate) fn handle(&self) -> &framebuffer::Handle {
        &self.handle
    }

    /// Returns the exact format registered with KMS.
    pub(crate) fn format(&self) -> Format {
        self.format
    }
}

impl AsRef<framebuffer::Handle> for StrictGbmFramebuffer {
    fn as_ref(&self) -> &framebuffer::Handle {
        &self.handle
    }
}

impl Drop for StrictGbmFramebuffer {
    fn drop(&mut self) {
        if let Err(error) = self.drm.destroy_framebuffer(self.handle) {
            tracing::warn!(
                framebuffer = ?self.handle,
                %error,
                "Failed to destroy strict GBM KMS framebuffer"
            );
        }
    }
}

/// Keeps the GBM allocation alive until its KMS framebuffer has been destroyed.
///
/// Field order is intentional: Rust drops fields in declaration order.
pub(crate) struct GbmCpuUploadFramebuffer {
    framebuffer: StrictGbmFramebuffer,
    buffer: GbmCpuUploadBuffer,
    id: DrmFramebufferId,
}

impl GbmCpuUploadFramebuffer {
    /// Registers the allocation and retains both resources in one owner.
    pub(crate) fn register(
        id: DrmFramebufferId,
        drm: &DrmDeviceFd,
        buffer: GbmCpuUploadBuffer,
    ) -> Result<Self, GbmFramebufferRegistrationError> {
        let framebuffer = StrictGbmFramebuffer::register(drm, &buffer)?;

        Ok(Self {
            framebuffer,
            buffer,
            id,
        })
    }

    /// Returns the compositor identity used by the per-output lifecycle.
    pub(crate) fn id(&self) -> DrmFramebufferId {
        self.id
    }

    /// Borrows the strict KMS framebuffer.
    pub(crate) fn framebuffer(&self) -> &StrictGbmFramebuffer {
        &self.framebuffer
    }

    /// Borrows the CPU-upload allocation without transferring ownership.
    pub(crate) fn buffer(&self) -> &GbmCpuUploadBuffer {
        &self.buffer
    }

    /// Mutably borrows the CPU-upload allocation before KMS submission.
    pub(crate) fn buffer_mut(&mut self) -> &mut GbmCpuUploadBuffer {
        &mut self.buffer
    }
}
