// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Owns CPU-upload GBM framebuffers and their per-output KMS lifecycle.

use std::{collections::HashMap, error::Error, fmt};

use smithay::{
    backend::{allocator::gbm::GbmDevice, drm::DrmDeviceFd},
    reexports::drm::control::framebuffer,
};

use super::{
    gbm_buffer_allocator::{GbmCpuUploadAllocationError, GbmCpuUploadAllocator},
    gbm_buffer_writer::GbmBufferWriterError,
    gbm_framebuffer::{GbmCpuUploadFramebuffer, GbmFramebufferRegistrationError},
    presentation::{DrmFramebufferId, DrmPresentationError, DrmPresentationLifecycle},
};

/// Errors from allocation, writing and KMS lifecycle management.
#[derive(Debug)]
pub(crate) enum GbmCpuFramebufferPoolError {
    IdentifierExhausted,
    UnknownFramebuffer(DrmFramebufferId),
    FramebufferStillInUse(DrmFramebufferId),
    Allocation(GbmCpuUploadAllocationError),
    Registration(GbmFramebufferRegistrationError),
    Write(GbmBufferWriterError),
    Presentation(DrmPresentationError),
}

impl fmt::Display for GbmCpuFramebufferPoolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IdentifierExhausted => {
                formatter.write_str("GBM framebuffer identifiers exhausted")
            }
            Self::UnknownFramebuffer(id) => {
                write!(formatter, "Unknown GBM framebuffer {}", id.value())
            }
            Self::FramebufferStillInUse(id) => write!(
                formatter,
                "GBM framebuffer {} is active, pending or in a KMS transaction",
                id.value()
            ),
            Self::Allocation(error) => write!(formatter, "GBM allocation failed: {error}"),
            Self::Registration(error) => write!(formatter, "KMS registration failed: {error}"),
            Self::Write(error) => write!(formatter, "GBM upload failed: {error}"),
            Self::Presentation(error) => write!(formatter, "KMS lifecycle failed: {error}"),
        }
    }
}

impl Error for GbmCpuFramebufferPoolError {}

/// Owns all CPU-upload framebuffers for exactly one KMS output.
pub(crate) struct GbmCpuFramebufferPool {
    next_id: u64,
    buffers: HashMap<DrmFramebufferId, GbmCpuUploadFramebuffer>,
    lifecycle: DrmPresentationLifecycle,
}

impl Default for GbmCpuFramebufferPool {
    fn default() -> Self {
        Self::new()
    }
}

impl GbmCpuFramebufferPool {
    /// Creates an empty pool for one output.
    pub(crate) fn new() -> Self {
        Self {
            next_id: 1,
            buffers: HashMap::new(),
            lifecycle: DrmPresentationLifecycle::new(),
        }
    }

    /// Allocates, registers and retains one CPU-upload framebuffer.
    pub(crate) fn allocate(
        &mut self,
        gbm: &GbmDevice<DrmDeviceFd>,
        drm: &DrmDeviceFd,
        width: u32,
        height: u32,
        plane_formats: &[smithay::backend::allocator::Fourcc],
    ) -> Result<DrmFramebufferId, GbmCpuFramebufferPoolError> {
        let id = DrmFramebufferId::new(self.next_id);
        let next_id = self
            .next_id
            .checked_add(1)
            .ok_or(GbmCpuFramebufferPoolError::IdentifierExhausted)?;

        let allocator = GbmCpuUploadAllocator::new(gbm);
        let buffer = allocator
            .allocate(width, height, plane_formats)
            .map_err(GbmCpuFramebufferPoolError::Allocation)?;
        let framebuffer = GbmCpuUploadFramebuffer::register(drm, buffer)
            .map_err(GbmCpuFramebufferPoolError::Registration)?;

        self.buffers.insert(id, framebuffer);
        self.next_id = next_id;
        Ok(id)
    }

    /// Uploads pixels only when KMS does not own or await this framebuffer.
    pub(crate) fn write_rgba8(
        &mut self,
        id: DrmFramebufferId,
        pixels: &[u8],
        width: u32,
        height: u32,
    ) -> Result<(), GbmCpuFramebufferPoolError> {
        if self.is_in_use(id) {
            return Err(GbmCpuFramebufferPoolError::FramebufferStillInUse(id));
        }
        let buffer = self
            .buffers
            .get_mut(&id)
            .ok_or(GbmCpuFramebufferPoolError::UnknownFramebuffer(id))?;
        buffer
            .buffer_mut()
            .write_rgba8(pixels, width, height)
            .map_err(GbmCpuFramebufferPoolError::Write)
    }

    /// Borrows the framebuffer handle for atomic plane configuration.
    pub(crate) fn framebuffer(
        &self,
        id: DrmFramebufferId,
    ) -> Result<&framebuffer::Handle, GbmCpuFramebufferPoolError> {
        self.buffers
            .get(&id)
            .map(|buffer| buffer.framebuffer().handle())
            .ok_or(GbmCpuFramebufferPoolError::UnknownFramebuffer(id))
    }

    /// Begins atomic TEST_ONLY validation for a candidate.
    pub(crate) fn begin_test(
        &mut self,
        id: DrmFramebufferId,
    ) -> Result<(), GbmCpuFramebufferPoolError> {
        self.ensure_registered(id)?;
        self.lifecycle
            .begin_test(id)
            .map_err(GbmCpuFramebufferPoolError::Presentation)
    }

    /// Records successful atomic TEST_ONLY validation.
    pub(crate) fn test_passed(&mut self) -> Result<(), GbmCpuFramebufferPoolError> {
        self.lifecycle
            .test_passed()
            .map_err(GbmCpuFramebufferPoolError::Presentation)
    }

    /// Cancels a rejected TEST_ONLY request.
    pub(crate) fn test_failed(&mut self) -> Result<DrmFramebufferId, GbmCpuFramebufferPoolError> {
        self.lifecycle
            .test_failed()
            .map_err(GbmCpuFramebufferPoolError::Presentation)
    }

    /// Records an accepted non-blocking KMS commit.
    pub(crate) fn commit_accepted(&mut self) -> Result<(), GbmCpuFramebufferPoolError> {
        self.lifecycle
            .commit_accepted()
            .map_err(GbmCpuFramebufferPoolError::Presentation)
    }

    /// Cancels a rejected KMS commit.
    pub(crate) fn commit_failed(&mut self) -> Result<DrmFramebufferId, GbmCpuFramebufferPoolError> {
        self.lifecycle
            .commit_failed()
            .map_err(GbmCpuFramebufferPoolError::Presentation)
    }

    /// Completes a page flip and returns the framebuffer released by KMS.
    pub(crate) fn page_flip_complete(
        &mut self,
        displayed: DrmFramebufferId,
    ) -> Result<Option<DrmFramebufferId>, GbmCpuFramebufferPoolError> {
        self.lifecycle
            .page_flip_complete(displayed)
            .map_err(GbmCpuFramebufferPoolError::Presentation)
    }

    /// Returns the framebuffer awaiting its page-flip event.
    pub(crate) fn pending(&self) -> Option<DrmFramebufferId> {
        self.lifecycle.pending_candidate()
    }

    /// Releases all CPU-upload buffers after the owning KMS output is confirmed disabled.
    ///
    /// A pending atomic transaction or page flip prevents cleanup. On success,
    /// the kernel no longer scans out the current buffer, so userspace may drop
    /// every GBM allocation and framebuffer registration.
    pub(crate) fn output_disabled(&mut self) -> Result<usize, GbmCpuFramebufferPoolError> {
        self.lifecycle
            .output_disabled()
            .map_err(GbmCpuFramebufferPoolError::Presentation)?;
        let retired = self.buffers.len();
        self.buffers.clear();
        Ok(retired)
    }

    /// Destroys a framebuffer only after KMS has released it.
    pub(crate) fn retire(
        &mut self,
        id: DrmFramebufferId,
    ) -> Result<(), GbmCpuFramebufferPoolError> {
        if self.is_in_use(id) {
            return Err(GbmCpuFramebufferPoolError::FramebufferStillInUse(id));
        }
        self.buffers
            .remove(&id)
            .map(|_| ())
            .ok_or(GbmCpuFramebufferPoolError::UnknownFramebuffer(id))
    }

    fn ensure_registered(&self, id: DrmFramebufferId) -> Result<(), GbmCpuFramebufferPoolError> {
        self.buffers
            .contains_key(&id)
            .then_some(())
            .ok_or(GbmCpuFramebufferPoolError::UnknownFramebuffer(id))
    }

    fn is_in_use(&self, id: DrmFramebufferId) -> bool {
        self.lifecycle.current() == Some(id)
            || self.lifecycle.pending_candidate() == Some(id)
            || self.lifecycle.transaction_candidate() == Some(id)
    }
}

impl Drop for GbmCpuFramebufferPool {
    fn drop(&mut self) {
        let has_kms_owned_buffers = self
            .buffers
            .keys()
            .copied()
            .any(|id| self.is_in_use(id));
        if has_kms_owned_buffers {
            // A failed output disable or unresolved page flip means KMS may
            // still reference these GBM allocations. Preserve them rather
            // than closing GEM/GBM owners underneath an active scanout.
            std::mem::forget(std::mem::take(&mut self.buffers));
        }
    }
}
