// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Owns and manages the native libseat session.

use std::{error::Error, fmt};

use smithay::backend::session::{
    Session,
    libseat::{LibSeatSession, LibSeatSessionNotifier},
};

/// Error returned while initializing or managing the DRM session.
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
            DrmSessionError::new(format!("Failed to open a libseat session: {error}"))
        })?;

        Ok(Self {
            session,
            notifier: Some(notifier),
        })
    }

    /// Returns whether the native session is currently active.
    pub fn is_active(&self) -> bool {
        self.session.is_active()
    }

    /// Returns the name of the seat associated with this session.
    pub fn seat(&self) -> String {
        self.session.seat()
    }

    /// Returns mutable access to the session handle used to open devices.
    pub fn session(&mut self) -> &mut LibSeatSession {
        &mut self.session
    }

    /// Returns a shared libseat handle for input backends that open devices
    /// through the same active seat session.
    pub fn session_handle(&self) -> LibSeatSession {
        self.session.clone()
    }

    /// Transfers the notifier to the Calloop event loop.
    ///
    /// This can only be called once.
    pub fn take_notifier(&mut self) -> Result<LibSeatSessionNotifier, DrmSessionError> {
        self.notifier
            .take()
            .ok_or_else(|| DrmSessionError::new("The libseat notifier was already transferred"))
    }
}
