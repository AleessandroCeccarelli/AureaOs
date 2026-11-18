// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Capture Core
// Copyright (C) 2026 Alessandro Ceccarelli

//! Capture coordinator errors.

/// Errors returned by capture request validation and session transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureError {
    InvalidRegion,
    PermissionDenied,
    SessionBusy,
    NoActiveSession,
    SessionIdExhausted,
    InvalidTransition,
}

impl std::fmt::Display for CaptureError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidRegion => "Capture region must have a non-zero size",
            Self::PermissionDenied => "Capture permission was not granted for every requested source",
            Self::SessionBusy => "Another capture session is already active",
            Self::NoActiveSession => "The requested capture session is not active",
            Self::SessionIdExhausted => "Capture session identifier space is exhausted",
            Self::InvalidTransition => "Capture session transition is not allowed",
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for CaptureError {}
