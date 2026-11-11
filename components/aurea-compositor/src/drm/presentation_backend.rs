// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Executes the KMS side of a Vulkan framebuffer presentation.
//!
//! This adapter is deliberately independent from output discovery and frame
//! scheduling. The caller supplies the primary-plane state and keeps both
//! the candidate and current framebuffer objects alive until their respective
//! KMS and GPU ownership conditions have completed.

use std::{error::Error, fmt};

use smithay::backend::drm::{
    crtc,
    DrmNode,
    DrmSurface,
    PlaneState,
};

use super::presentation::{
    DrmFramebufferId,
    DrmPresentationError,
    DrmPresentationLifecycle,
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
    Lifecycle(DrmPresentationError),
}

impl fmt::Display for DrmPresentationBackendError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LegacySurfaceUnsupported => formatter.write_str(
                "Vulkan presentation requires atomic DRM TEST_ONLY support",
            ),
            Self::MissingActivePlane => formatter.write_str(
                "Vulkan presentation requires at least one configured DRM plane",
            ),
            Self::MissingInFence => formatter.write_str(
                "Vulkan KMS presentation requires a sync-file fence on every active plane",
            ),
            Self::UnexpectedCrtc => formatter.write_str(
                "DRM vblank event does not belong to this presentation surface",
            ),
            Self::KmsOperation(message) => write!(formatter, "KMS presentation failed: {message}"),
            Self::Lifecycle(error) => write!(formatter, "Invalid KMS presentation lifecycle: {error}"),
        }
    }
}

impl Error for DrmPresentationBackendError {}

impl From<DrmPresentationError> for DrmPresentationBackendError {
    fn from(error: DrmPresentationError) -> Self {
        Self::Lifecycle(error)
    }
}

/// Owns the per-CRTC presentation state for one Smithay DRM surface.
#[derive(Debug)]
pub struct DrmPresentationBackend {
    node: DrmNode,
    crtc: crtc::Handle,
    lifecycle: DrmPresentationLifecycle,
}

impl DrmPresentationBackend {
    /// Creates a presentation adapter for the exact DRM node and CRTC.
    pub fn new(node: DrmNode, surface: &DrmSurface) -> Self {
        Self {
            node,
            crtc: surface.crtc(),
            lifecycle: DrmPresentationLifecycle::new(),
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
        self.lifecycle.current()
    }

    /// Returns the candidate awaiting its KMS completion event.
    pub fn pending(&self) -> Option<DrmFramebufferId> {
        self.lifecycle.pending_candidate()
    }

    /// Tests and submits one primary-plane state.
    ///
    /// Atomic TEST_ONLY is mandatory. The initial/pending modeset uses
    /// Smithay's `commit`; steady-state frames use `page_flip`. Both ask
    /// Smithay to emit a completion event. The supplied plane states are
    /// cloned so the exact same state is validated and then submitted.
    pub fn submit(
        &mut self,
        surface: &DrmSurface,
        candidate: DrmFramebufferId,
        planes: &[PlaneState<'_>],
    ) -> Result<(), DrmPresentationBackendError> {
        if surface.is_legacy() {
            return Err(DrmPresentationBackendError::LegacySurfaceUnsupported);
        }
        if surface.crtc() != self.crtc {
            return Err(DrmPresentationBackendError::UnexpectedCrtc);
        }

        let active_planes = planes
            .iter()
            .filter_map(|plane| plane.config.as_ref())
            .collect::<Vec<_>>();

        if active_planes.is_empty() {
            return Err(DrmPresentationBackendError::MissingActivePlane);
        }
        if active_planes.iter().any(|config| config.fence.is_none()) {
            return Err(DrmPresentationBackendError::MissingInFence);
        }

        let requires_modeset = surface.commit_pending();
        self.lifecycle.begin_test(candidate)?;

        if let Err(error) = surface.test_state(planes.iter().cloned(), requires_modeset) {
            let _ = self.lifecycle.test_failed();
            return Err(DrmPresentationBackendError::KmsOperation(error.to_string()));
        }

        self.lifecycle.test_passed()?;

        let submission = if requires_modeset {
            surface.commit(planes.iter().cloned(), true)
        } else {
            surface.page_flip(planes.iter().cloned(), true)
        };

        if let Err(error) = submission {
            let _ = self.lifecycle.commit_failed();
            return Err(DrmPresentationBackendError::KmsOperation(error.to_string()));
        }

        self.lifecycle.commit_accepted()?;
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
            .lifecycle
            .pending_candidate()
            .ok_or(DrmPresentationBackendError::Lifecycle(
                DrmPresentationError::NoTransaction,
            ))?;

        self.lifecycle
            .page_flip_complete(displayed)
            .map_err(Into::into)
    }
}
