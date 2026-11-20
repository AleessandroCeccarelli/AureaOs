// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Presents completed CPU-uploaded GBM framebuffers through DRM/KMS.

use std::{error::Error, fmt};

use smithay::{
    backend::drm::{DrmNode, DrmSurface, PlaneState},
    reexports::drm::control::crtc,
};

use super::presentation::{DrmFramebufferId, DrmPresentationError, DrmPresentationLifecycle};

/// Errors returned by the CPU-upload KMS presenter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CpuPresentationError {
    LegacySurfaceUnsupported,
    MissingActivePlane,
    UnexpectedCrtc,
    KmsOperation(String),
    Lifecycle(DrmPresentationError),
}

impl fmt::Display for CpuPresentationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LegacySurfaceUnsupported => {
                formatter.write_str("CPU presentation requires atomic DRM TEST_ONLY support")
            }
            Self::MissingActivePlane => {
                formatter.write_str("CPU presentation requires at least one configured DRM plane")
            }
            Self::UnexpectedCrtc => {
                formatter.write_str("DRM event does not belong to this presentation surface")
            }
            Self::KmsOperation(message) => write!(formatter, "KMS presentation failed: {message}"),
            Self::Lifecycle(error) => write!(formatter, "Invalid CPU presentation lifecycle: {error}"),
        }
    }
}

impl Error for CpuPresentationError {}

impl From<DrmPresentationError> for CpuPresentationError {
    fn from(error: DrmPresentationError) -> Self {
        Self::Lifecycle(error)
    }
}

/// Tracks KMS ownership for CPU-uploaded buffers on exactly one CRTC.
///
/// The caller must finish the GBM buffer write and any required CPU cache
/// synchronization before submitting its framebuffer. This presenter owns
/// only lifecycle state; the GBM pool retains the actual buffer and FB.
#[derive(Debug)]
pub struct CpuPresentationBackend {
    node: DrmNode,
    crtc: crtc::Handle,
    lifecycle: DrmPresentationLifecycle,
}

impl CpuPresentationBackend {
    /// Creates a presenter bound to the DRM node and CRTC of a surface.
    pub fn new(node: DrmNode, surface: &DrmSurface) -> Self {
        Self {
            node,
            crtc: surface.crtc(),
            lifecycle: DrmPresentationLifecycle::new(),
        }
    }

    /// Returns the DRM node associated with this output.
    pub fn node(&self) -> DrmNode {
        self.node
    }

    /// Returns the CRTC associated with this output.
    pub fn crtc(&self) -> crtc::Handle {
        self.crtc
    }

    /// Returns the framebuffer currently scanned out.
    pub fn current(&self) -> Option<DrmFramebufferId> {
        self.lifecycle.current()
    }

    /// Returns the framebuffer awaiting its page-flip event.
    pub fn pending(&self) -> Option<DrmFramebufferId> {
        self.lifecycle.pending_candidate()
    }

    /// Submits a CPU-uploaded frame after its buffer write has completed.
    ///
    /// No Vulkan ownership or sync-file contract is involved. Atomic TEST_ONLY
    /// is still mandatory, and the framebuffer must remain owned by its pool
    /// until this presenter reports it released after a page flip.
    pub fn submit_after_upload(
        &mut self,
        surface: &DrmSurface,
        candidate: DrmFramebufferId,
        planes: &[PlaneState<'_>],
    ) -> Result<(), CpuPresentationError> {
        if surface.is_legacy() {
            return Err(CpuPresentationError::LegacySurfaceUnsupported);
        }
        if surface.crtc() != self.crtc {
            return Err(CpuPresentationError::UnexpectedCrtc);
        }
        if !planes.iter().any(|plane| plane.config.is_some()) {
            return Err(CpuPresentationError::MissingActivePlane);
        }

        self.lifecycle.begin_test(candidate)?;
        let requires_modeset = surface.commit_pending();

        if let Err(error) = surface.test_state(planes.iter().cloned(), requires_modeset) {
            let _ = self.lifecycle.test_failed();
            return Err(CpuPresentationError::KmsOperation(error.to_string()));
        }

        self.lifecycle.test_passed()?;
        let submission = if requires_modeset {
            surface.commit(planes.iter().cloned(), true)
        } else {
            surface.page_flip(planes.iter().cloned(), true)
        };

        if let Err(error) = submission {
            let _ = self.lifecycle.commit_failed();
            return Err(CpuPresentationError::KmsOperation(error.to_string()));
        }

        self.lifecycle.commit_accepted()?;
        Ok(())
    }

    /// Completes the pending flip and returns the framebuffer released by KMS.
    pub fn on_vblank(
        &mut self,
        node: DrmNode,
        crtc: crtc::Handle,
    ) -> Result<Option<DrmFramebufferId>, CpuPresentationError> {
        if node != self.node || crtc != self.crtc {
            return Err(CpuPresentationError::UnexpectedCrtc);
        }
        let Some(displayed) = self.pending() else {
            return Ok(None);
        };
        self.lifecycle
            .page_flip_complete(displayed)
            .map_err(Into::into)
    }
}
