// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Registers CPU-upload GBM allocations as KMS framebuffers.

use std::{error::Error, fmt};

use smithay::backend::drm::{
    DrmDeviceFd,
    gbm::{GbmFramebuffer, framebuffer_from_bo},
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

/// Keeps a CPU-upload buffer alive for the complete KMS framebuffer lifetime.
///
/// Field order is intentional: Rust drops fields in declaration order, so
/// the KMS framebuffer is removed before its backing GBM allocation is freed.
pub(crate) struct GbmCpuUploadFramebuffer {
    framebuffer: GbmFramebuffer,
    buffer: GbmCpuUploadBuffer,
    id: DrmFramebufferId,
}

impl GbmCpuUploadFramebuffer {
    /// Registers the allocation on the DRM device and retains both resources.
    pub(crate) fn register(
        id: DrmFramebufferId,
        drm: &DrmDeviceFd,
        buffer: GbmCpuUploadBuffer,
    ) -> Result<Self, GbmFramebufferRegistrationError> {
        let framebuffer = framebuffer_from_bo(drm, buffer.gbm_buffer(), false)
            .map_err(|error| GbmFramebufferRegistrationError(error.to_string()))?;

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

    /// Borrows the Smithay framebuffer for KMS plane configuration.
    pub(crate) fn framebuffer(&self) -> &GbmFramebuffer {
        &self.framebuffer
    }

    /// Borrows the CPU-upload allocation without transferring its ownership.
    pub(crate) fn buffer(&self) -> &GbmCpuUploadBuffer {
        &self.buffer
    }

    /// Mutably borrows the CPU-upload allocation before it is submitted to KMS.
    pub(crate) fn buffer_mut(&mut self) -> &mut GbmCpuUploadBuffer {
        &mut self.buffer
    }
}
