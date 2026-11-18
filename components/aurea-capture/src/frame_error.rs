// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Capture Core
// Copyright (C) 2026 Alessandro Ceccarelli

//! Validation errors for owned capture frames.

/// Invalid pixel storage or dimensions supplied by a capture backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureFrameError {
    InvalidDimensions,
    InvalidStride,
    InvalidBufferLength,
    SizeOverflow,
}

impl std::fmt::Display for CaptureFrameError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidDimensions => "Capture frame dimensions must be non-zero",
            Self::InvalidStride => "Capture frame stride is smaller than one pixel row",
            Self::InvalidBufferLength => "Capture frame byte length does not match its stride",
            Self::SizeOverflow => "Capture frame byte size overflowed",
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for CaptureFrameError {}
