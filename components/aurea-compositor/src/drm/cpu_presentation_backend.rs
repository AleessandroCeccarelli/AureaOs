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

use super::{
    gbm_framebuffer_pool::{GbmCpuFramebufferPool, GbmCpuFramebufferPoolError},
    presentation::DrmFramebufferId,
};

/// Errors returned by the CPU-upload KMS presenter.
#[derive(Debug)]
pub enum CpuPresentationError {
    LegacySurfaceUnsupported,
    MissingActivePlane,
    CandidateFramebufferNotConfigured,
    UnexpectedCrtc,
    KmsOperation(String),
    Lifecycle(GbmCpuFramebufferPoolError),
}

impl fmt::Display for CpuPresentationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LegacySurfaceUnsupported => formatter.write_str(
                "CPU presentation requires atomic DRM TEST_ONLY support",
            ),
            Self::MissingActivePlane => formatter.write_str(
                "CPU presentation requires the output primary plane to be configured",
            ),
            Self::CandidateFramebufferNotConfigured => formatter.write_str(
                "The configured primary plane does not reference the submitted CPU framebuffer",
            ),
            Self::UnexpectedCrtc => formatter.write_str(
                "DRM event does not belong to this presentation surface",
            ),
            Self::KmsOperation(message) => write!(formatter, "KMS presentation failed: {message}"),
            Self::Lifecycle(error) => write!(formatter, "CPU framebuffer lifecycle failed: {error}"),
        }
    }
}

impl Error for CpuPresentationError {}

impl From<GbmCpuFramebufferPoolError> for CpuPresentationError {
    fn from(error: GbmCpuFramebufferPoolError) -> Self {
        Self::Lifecycle(error)
    }
}

/// Submits CPU-uploaded framebuffers for one DRM node and CRTC.
///
/// The caller must finish the GBM write and required CPU cache synchronization
/// before submission. The pool owns the GBM allocation and KMS framebuffer.
#[derive(Debug)]
pub struct CpuPresentationBackend {
    node: DrmNode,
    crtc: crtc::Handle,
}

impl CpuPresentationBackend {
    /// Creates a presenter bound to the DRM node and CRTC of a surface.
    pub fn new(node: DrmNode, surface: &DrmSurface) -> Self {
        Self { node, crtc: surface.crtc() }
    }

    /// Returns the DRM node associated with this output.
    pub fn node(&self) -> DrmNode {
        self.node
    }

    /// Returns the CRTC associated with this output.
    pub fn crtc(&self) -> crtc::Handle {
        self.crtc
    }

    /// Submits a CPU-uploaded frame after its buffer write has completed.
    ///
    /// Atomic TEST_ONLY is mandatory. The pool lifecycle prevents a candidate
    /// from being overwritten or destroyed while KMS owns it.
    pub fn submit_after_upload(
        &mut self,
        surface: &DrmSurface,
        pool: &mut GbmCpuFramebufferPool,
        candidate: DrmFramebufferId,
        planes: &[PlaneState<'_>],
    ) -> Result<(), CpuPresentationError> {
        if surface.is_legacy() {
            return Err(CpuPresentationError::LegacySurfaceUnsupported);
        }
        if surface.crtc() != self.crtc {
            return Err(CpuPresentationError::UnexpectedCrtc);
        }
        let expected_framebuffer = pool.framebuffer(candidate)?;
        let primary_plane = surface.plane();
        let candidate_is_primary = planes.iter().any(|plane| {
            plane.handle == primary_plane
                && plane
                    .config
                    .as_ref()
                    .is_some_and(|config| config.fb == *expected_framebuffer)
        });
        if !candidate_is_primary {
            return Err(CpuPresentationError::CandidateFramebufferNotConfigured);
        }

        pool.begin_test(candidate)?;
        let requires_modeset = surface.commit_pending();

        if let Err(error) = surface.test_state(planes.iter().cloned(), requires_modeset) {
            let _ = pool.test_failed();
            return Err(CpuPresentationError::KmsOperation(error.to_string()));
        }

        pool.test_passed()?;
        let submission = if requires_modeset {
            surface.commit(planes.iter().cloned(), true)
        } else {
            surface.page_flip(planes.iter().cloned(), true)
        };

        if let Err(error) = submission {
            let _ = pool.commit_failed();
            return Err(CpuPresentationError::KmsOperation(error.to_string()));
        }

        pool.commit_accepted()?;
        Ok(())
    }

    /// Completes the pending flip and returns the framebuffer released by KMS.
    pub fn on_vblank(
        &mut self,
        node: DrmNode,
        crtc: crtc::Handle,
        pool: &mut GbmCpuFramebufferPool,
    ) -> Result<Option<DrmFramebufferId>, CpuPresentationError> {
        if node != self.node || crtc != self.crtc {
            return Err(CpuPresentationError::UnexpectedCrtc);
        }
        let Some(displayed) = pool.pending() else {
            return Ok(None);
        };
        pool.page_flip_complete(displayed).map_err(Into::into)
    }
}
