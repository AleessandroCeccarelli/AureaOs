// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Capture Core
// Copyright (C) 2026 Alessandro Ceccarelli

//! Technology-independent capture session coordination.
//!
//! This crate deliberately contains no Wayland, Vulkan, PipeWire or encoder
//! dependencies. Platform adapters belong in separate modules or services.

mod coordinator;
mod error;
mod frame;
mod frame_error;
mod frame_queue;
mod queue_error;
mod request;
mod session;

pub use coordinator::CaptureCoordinator;
pub use error::CaptureError;
pub use request::{
    AudioCaptureMode, CaptureKind, CapturePermissionGrant, CaptureRegion, CaptureRequest,
    CaptureTarget, DisplayId, WindowId,
};
pub use session::{CaptureSession, CaptureSessionId, CaptureSessionState};

pub use frame::{CaptureFrame, CapturePixelFormat, CaptureTimestamp};
pub use frame_error::CaptureFrameError;
pub use frame_queue::CaptureFrameQueue;
pub use queue_error::CaptureQueueError;
