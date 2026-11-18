// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Capture Core
// Copyright (C) 2026 Alessandro Ceccarelli

//! Capture session identity, state and immutable request association.

use crate::{CaptureKind, CaptureRequest, CaptureTarget};

/// Monotonically assigned identifier for one capture session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CaptureSessionId(u64);

impl CaptureSessionId {
    pub(crate) const fn from_value(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Lifecycle states shared by screenshot and recording workflows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureSessionState {
    AwaitingAuthorization,
    Ready,
    Capturing,
    Paused,
    Finalizing,
    Completed,
    Cancelled,
    Failed,
}

impl CaptureSessionState {
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled | Self::Failed)
    }
}

/// One coordinated capture operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureSession {
    id: CaptureSessionId,
    request: CaptureRequest,
    state: CaptureSessionState,
}

impl CaptureSession {
    pub(crate) const fn new(id: CaptureSessionId, request: CaptureRequest) -> Self {
        Self {
            id,
            request,
            state: CaptureSessionState::AwaitingAuthorization,
        }
    }

    pub const fn id(self) -> CaptureSessionId { self.id }
    pub const fn request(self) -> CaptureRequest { self.request }
    pub const fn kind(self) -> CaptureKind { self.request.kind() }
    pub const fn target(self) -> CaptureTarget { self.request.target() }
    pub const fn state(self) -> CaptureSessionState { self.state }

    pub(crate) fn set_state(&mut self, state: CaptureSessionState) {
        self.state = state;
    }
}
