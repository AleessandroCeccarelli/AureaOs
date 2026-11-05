
 // SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Native DRM/KMS backend session management.

use std::{error::Error, fmt};

use smithay::backend::session::libseat::{
    LibSeatSession,
    LibSeatSessionNotifier,
};

/// Error returned while initializing the native DRM session.
#[derive(Debug)]
pub struct DrmSessionError {
    message: String,
}

impl DrmSessionError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for DrmSessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for DrmSessionError {}

/// Owns the libseat session and its event notifier.
pub struct DrmSession {
    session: LibSeatSession,
    notifier: Option<LibSeatSessionNotifier>,
}

impl DrmSession {
    /// Opens a session through libseat.
    pub fn new() -> Result<Self, DrmSessionError> {
        let (session, notifier) = LibSeatSession::new().map_err(|error| {
            DrmSessionError::new(format!(
                "Failed to open a libseat session: {error}"
            ))
        })?;

        Ok(Self {
            session,
            notifier: Some(notifier),
        })
    }

    /// Returns the session handle used to open privileged devices.
    pub fn session(&self) -> &LibSeatSession {
        &self.session
    }

    /// Transfers the notifier to the Calloop event loop.
    ///
    /// This can only be called once.
    pub fn take_notifier(
        &mut self,
    ) -> Result<LibSeatSessionNotifier, DrmSessionError> {
        self.notifier.take().ok_or_else(|| {
            DrmSessionError::new("The libseat notifier was already transferred")
        })
    }
}
pub mod device;
