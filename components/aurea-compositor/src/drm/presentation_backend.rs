// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Executes the KMS side of a Vulkan framebuffer presentation.
//!
//! The adapter owns one output's frame-lifecycle coordinator while leaving
//! framebuffer object storage and render scheduling to their respective
//! owners. The caller supplies primary-plane state and keeps candidate and
//! current framebuffer objects alive until KMS and GPU ownership are resolved.

use std::{error::Error, fmt};

use smithay::{
    backend::drm::{DrmNode, DrmSurface, PlaneState},
    reexports::drm::control::crtc,
};

use super::{
    frame_coordinator::{DrmOutputFrameCoordinator, DrmOutputFrameError},
    presentation::DrmFramebufferId,
};

/// Errors returned while submitting or completing a KMS presentation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrmPresentationBackendError {
    /// Atomic TEST_ONLY is required; legacy KMS cannot provide that guarantee.
    LegacySurfaceUnsupported,
    /// Every active Vulkan plane must provide a sync-file IN_FENCE_FD.
    MissingActivePlane,
    /// Every active Vulkan plane must provide a sync-file IN_FENCE_FD.
    MissingInFence,
    /// The event belongs to a different DRM device or CRTC.
    UnexpectedCrtc,
    /// Smithay rejected the KMS operation.
    KmsOperation(String),
    /// The lifecycle tracker rejected an invalid transition.
    Lifecycle(DrmOutputFrameError),
}

impl fmt::Display for DrmPresentationBackendError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LegacySurfaceUnsupported => {
                formatter.write_str("Vulkan presentation requires atomic DRM TEST_ONLY support")
            }
            Self::MissingActivePlane => formatter
                .write_str("Vulkan presentation requires at least one configured DRM plane"),
            Self::MissingInFence => formatter.write_str(
                "Vulkan KMS presentation requires a sync-file fence on every active plane",
            ),
            Self::UnexpectedCrtc => {
                formatter.write_str("DRM vblank event does not belong to this presentation surface")
            }
            Self::KmsOperation(message) => write!(formatter, "KMS presentation failed: {message}"),
            Self::Lifecycle(error) => {
                write!(formatter, "Invalid KMS presentation lifecycle: {error}")
            }
        }
    }
}

impl Error for DrmPresentationBackendError {}

impl From<DrmOutputFrameError> for DrmPresentationBackendError {
    fn from(error: DrmOutputFrameError) -> Self {
        Self::Lifecycle(error)
    }
}

/// Owns the per-CRTC presentation state for one Smithay DRM surface.
#[derive(Debug)]
pub struct DrmPresentationBackend {
    node: DrmNode,
    crtc: crtc::Handle,
    frames: DrmOutputFrameCoordinator,
}

impl DrmPresentationBackend {
    /// Creates a presentation adapter for the exact DRM node and CRTC.
    pub fn new(node: DrmNode, surface: &DrmSurface) -> Self {
        Self {
            node,
            crtc: surface.crtc(),
            frames: DrmOutputFrameCoordinator::new(),
        }
    }

    /// Returns the DRM device associated with this presentation surface.
    pub fn node(&self) -> DrmNode {
        self.node
    }

    /// Returns the CRTC associated with this presentation surface.
    pub fn crtc(&self) -> crtc::Handle {
        self.crtc
    }

    /// Returns the framebuffer currently known to be scanned out.
    pub fn current(&self) -> Option<DrmFramebufferId> {
        self.frames.current()
    }

    /// Returns the candidate awaiting its KMS completion event.
    pub fn pending(&self) -> Option<DrmFramebufferId> {
        self.frames.pending()
    }

    /// Registers a framebuffer before it can be submitted to this output.
    ///
    /// The caller must also drive the returned resource through rendering and
    /// GPU submission using frame_coordinator_mut before KMS presentation.
    pub fn register_framebuffer(
        &mut self,
        framebuffer: DrmFramebufferId,
    ) -> Result<crate::vulkan::VulkanFrameResourceId, DrmPresentationBackendError> {
        self.frames
            .register_framebuffer(framebuffer)
            .map_err(Into::into)
    }

    /// Accesses the per-output frame lifecycle for GPU completion and resource ownership.
    pub fn frame_coordinator_mut(&mut self) -> &mut DrmOutputFrameCoordinator {
        &mut self.frames
    }

    /// Validates the synchronization contract for a set of active DRM planes.
    ///
    /// Each boolean indicates whether the corresponding active plane has an
    /// IN_FENCE_FD. An empty set is never a valid presentation.
    pub fn validate_plane_synchronization(
        active_plane_fences: &[bool],
        require_in_fence: bool,
    ) -> Result<(), DrmPresentationBackendError> {
        if active_plane_fences.is_empty() {
            return Err(DrmPresentationBackendError::MissingActivePlane);
        }
        if require_in_fence && active_plane_fences.iter().any(|has_fence| !has_fence) {
            return Err(DrmPresentationBackendError::MissingInFence);
        }
        Ok(())
    }

    /// Tests and submits one primary-plane state using explicit IN_FENCE_FD synchronization.
    ///
    /// Every active plane must provide a sync-file fence. This is the preferred
    /// path when GPU work is submitted asynchronously.
    pub fn submit(
        &mut self,
        surface: &DrmSurface,
        candidate: DrmFramebufferId,
        planes: &[PlaneState<'_>],
    ) -> Result<(), DrmPresentationBackendError> {
        self.submit_with_sync_policy(surface, candidate, planes, true)
    }

    /// Tests and submits a frame after the caller has synchronously waited for
    /// all Vulkan work that writes its framebuffer to complete.
    ///
    /// This path intentionally permits planes without IN_FENCE_FD because the
    /// GPU-completion wait has already established that KMS can safely read
    /// the image. Do not use it with an in-flight Vulkan submission.
    pub fn submit_after_gpu_completion(
        &mut self,
        surface: &DrmSurface,
        candidate: DrmFramebufferId,
        planes: &[PlaneState<'_>],
    ) -> Result<(), DrmPresentationBackendError> {
        self.submit_with_sync_policy(surface, candidate, planes, false)
    }

    fn submit_with_sync_policy(
        &mut self,
        surface: &DrmSurface,
        candidate: DrmFramebufferId,
        planes: &[PlaneState<'_>],
        require_in_fence: bool,
    ) -> Result<(), DrmPresentationBackendError> {
        if surface.is_legacy() {
            return Err(DrmPresentationBackendError::LegacySurfaceUnsupported);
        }
        if surface.crtc() != self.crtc {
            return Err(DrmPresentationBackendError::UnexpectedCrtc);
        }

        let active_plane_fences = planes
            .iter()
            .filter_map(|plane| plane.config.as_ref())
            .map(|config| config.fence.is_some())
            .collect::<Vec<_>>();
        Self::validate_plane_synchronization(&active_plane_fences, require_in_fence)?;

        let requires_modeset = surface.commit_pending();
        self.frames.begin_presentation(candidate)?;

        if let Err(error) = surface.test_state(planes.iter().cloned(), requires_modeset) {
            let _ = self.frames.presentation_test_failed();
            return Err(DrmPresentationBackendError::KmsOperation(error.to_string()));
        }

        self.frames.presentation_test_passed()?;
        self.frames.validate_commit_ready(true)?;

        let submission = if requires_modeset {
            surface.commit(planes.iter().cloned(), true)
        } else {
            surface.page_flip(planes.iter().cloned(), true)
        };

        if let Err(error) = submission {
            let _ = self.frames.presentation_commit_failed();
            return Err(DrmPresentationBackendError::KmsOperation(error.to_string()));
        }

        self.frames.presentation_commit_accepted(true)?;
        Ok(())
    }

    /// Completes the outstanding presentation when its DRM vblank arrives.
    ///
    /// Returns the previously scanned-out framebuffer. The caller may retire
    /// it only after its independent Vulkan GPU-completion condition is met.
    pub fn on_vblank(
        &mut self,
        node: DrmNode,
        crtc: crtc::Handle,
    ) -> Result<Option<DrmFramebufferId>, DrmPresentationBackendError> {
        if node != self.node || crtc != self.crtc {
            return Err(DrmPresentationBackendError::UnexpectedCrtc);
        }

        let displayed = self
            .frames
            .pending()
            .ok_or(DrmPresentationBackendError::Lifecycle(
                DrmOutputFrameError::NoPendingFrame,
            ))?;

        self.frames
            .page_flip_complete(displayed)
            .map_err(Into::into)
    }
}
