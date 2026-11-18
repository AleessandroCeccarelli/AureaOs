// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Capture Core
// Copyright (C) 2026 Alessandro Ceccarelli

//! Validation errors for the capture frame queue.

/// Invalid frame queue configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureQueueError {
    ZeroCapacity,
    ZeroByteLimit,
}

impl std::fmt::Display for CaptureQueueError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::ZeroCapacity => "Capture frame queue capacity must be non-zero",
            Self::ZeroByteLimit => "Capture frame queue byte limit must be non-zero",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for CaptureQueueError {}
