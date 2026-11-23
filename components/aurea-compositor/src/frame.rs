// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Per-output frame scheduling and lifecycle management.

pub mod scheduler;
pub mod token;

pub use scheduler::FrameScheduler;
pub use token::{FrameLifecycleError, FrameRequestResult, FrameToken};
