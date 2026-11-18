// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Capture Core
// Copyright (C) 2026 Alessandro Ceccarelli

//! Validation errors for the capture frame queue.

/// Invalid frame queue configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureQueueError {
    ZeroCapacity,
}

impl std::fmt::Display for CaptureQueueError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Capture frame queue capacity must be non-zero")
    }
}

impl std::error::Error for CaptureQueueError {}
