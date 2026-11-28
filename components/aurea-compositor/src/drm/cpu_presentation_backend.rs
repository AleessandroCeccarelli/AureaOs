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
    CandidateFramebufferNotConfigured,
    UnexpectedCrtc,
    KmsOperation(String),
    Lifecycle(GbmCpuFramebufferPoolError),
    Recovery {
        operation: String,
        lifecycle: GbmCpuFramebufferPoolError,
    },
}

impl fmt::Display for CpuPresentationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LegacySurfaceUnsupported => {
                formatter.write_str("CPU presentation requires atomic DRM TEST_ONLY support")
            }
            Self::CandidateFramebufferNotConfigured => formatter.write_str(
                "The configured primary plane does not reference the submitted CPU framebuffer",
            ),
            Self::UnexpectedCrtc => {
                formatter.write_str("DRM event does not belong to this presentation surface")
            }
            Self::KmsOperation(message) => write!(formatter, "KMS presentation failed: {message}"),
            Self::Lifecycle(error) => {
                write!(formatter, "CPU framebuffer lifecycle failed: {error}")
            }
            Self::Recovery {
                operation,
                lifecycle,
            } => write!(
                formatter,
                "{operation}; CPU framebuffer lifecycle recovery failed: {lifecycle}"
            ),
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
    pending_mode_rollback: Option<smithay::reexports::drm::control::Mode>,
}

impl CpuPresentationBackend {
    /// Creates a presenter bound to the DRM node and CRTC of a surface.
    pub fn new(node: DrmNode, surface: &DrmSurface) -> Self {
        Self {
            node,
            crtc: surface.crtc(),
            pending_mode_rollback: None,
        }
    }

    /// Arms restoration of the active mode if the next CPU modeset presentation fails.
    pub fn arm_mode_rollback(&mut self, previous: smithay::reexports::drm::control::Mode) {
        self.pending_mode_rollback = Some(previous);
    }

    /// Cancels an armed mode rollback after the caller abandons the staged mode.
    pub fn cancel_mode_rollback(&mut self) {
        self.pending_mode_rollback = None;
    }

    fn rollback_mode_change(&mut self, surface: &DrmSurface) -> Result<(), String> {
        if let Some(previous) = self.pending_mode_rollback.take() {
            surface.use_mode(previous).map_err(|error| {
                self.pending_mode_rollback = Some(previous);
                error.to_string()
            })?;
        }
        Ok(())
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
            let _ = self.rollback_mode_change(surface);
            return Err(CpuPresentationError::LegacySurfaceUnsupported);
        }
        if surface.crtc() != self.crtc {
            let _ = self.rollback_mode_change(surface);
            return Err(CpuPresentationError::UnexpectedCrtc);
        }
        let expected_framebuffer = match pool.framebuffer(candidate) {
            Ok(framebuffer) => framebuffer,
            Err(error) => {
                let rollback = self.rollback_mode_change(surface);
                return Err(CpuPresentationError::KmsOperation(match rollback {
                    Ok(()) => format!("CPU framebuffer lookup failed: {error}"),
                    Err(restore) => format!(
                        "CPU framebuffer lookup failed: {error}; restoring previous KMS mode failed: {restore}"
                    ),
                }));
            }
        };
        let primary_plane = surface.plane();
        let candidate_is_primary = planes.iter().any(|plane| {
            plane.handle == primary_plane
                && plane
                    .config
                    .as_ref()
                    .is_some_and(|config| config.fb == *expected_framebuffer)
        });
        if !candidate_is_primary {
            let _ = self.rollback_mode_change(surface);
            return Err(CpuPresentationError::CandidateFramebufferNotConfigured);
        }

        if let Err(error) = pool.begin_test(candidate) {
            let rollback = self.rollback_mode_change(surface);
            return Err(CpuPresentationError::KmsOperation(match rollback {
                Ok(()) => format!("Unable to begin CPU framebuffer test: {error}"),
                Err(restore) => format!(
                    "Unable to begin CPU framebuffer test: {error}; restoring previous KMS mode failed: {restore}"
                ),
            }));
        }
        let requires_modeset = surface.commit_pending();

        if let Err(error) = surface.test_state(planes.iter().cloned(), requires_modeset) {
            let operation = format!("Atomic TEST_ONLY rejected the CPU framebuffer: {error}");
            let rollback = self.rollback_mode_change(surface);
            return match pool.test_failed() {
                Ok(_) => Err(CpuPresentationError::KmsOperation(match rollback {
                    Ok(()) => operation,
                    Err(restore) => {
                        format!("{operation}; restoring previous KMS mode failed: {restore}")
                    }
                })),
                Err(lifecycle) => Err(CpuPresentationError::Recovery {
                    operation,
                    lifecycle,
                }),
            };
        }

        if let Err(error) = pool.test_passed() {
            let rollback = self.rollback_mode_change(surface);
            return Err(CpuPresentationError::KmsOperation(match rollback {
                Ok(()) => {
                    format!("CPU framebuffer lifecycle rejected successful TEST_ONLY: {error}")
                }
                Err(restore) => format!(
                    "CPU framebuffer lifecycle rejected successful TEST_ONLY: {error}; restoring previous KMS mode failed: {restore}"
                ),
            }));
        }
        let submission = if requires_modeset {
            surface.commit(planes.iter().cloned(), true)
        } else {
            surface.page_flip(planes.iter().cloned(), true)
        };

        if let Err(error) = submission {
            let operation = format!("KMS rejected the CPU framebuffer commit: {error}");
            let rollback = self.rollback_mode_change(surface);
            return match pool.commit_failed() {
                Ok(_) => Err(CpuPresentationError::KmsOperation(match rollback {
                    Ok(()) => operation,
                    Err(restore) => {
                        format!("{operation}; restoring previous KMS mode failed: {restore}")
                    }
                })),
                Err(lifecycle) => Err(CpuPresentationError::Recovery {
                    operation,
                    lifecycle,
                }),
            };
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
        let released = pool
            .page_flip_complete(displayed)
            .map_err(CpuPresentationError::from)?;
        self.pending_mode_rollback = None;
        Ok(released)
    }
}
