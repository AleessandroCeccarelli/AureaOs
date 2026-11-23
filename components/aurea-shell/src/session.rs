// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Desktop session state and lifecycle.

use crate::error::ShellError;

/// Current state of the desktop session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Starting,
    Active,
    Locked,
    Stopping,
    Stopped,
}

/// Tracks and validates desktop session transitions.
pub struct DesktopSession {
    state: SessionState,
}

impl DesktopSession {
    /// Creates a new session in the starting state.
    pub fn new() -> Self {
        Self {
            state: SessionState::Starting,
        }
    }

    /// Returns the current session state.
    pub fn state(&self) -> SessionState {
        self.state
    }

    /// Activates a session after startup.
    pub fn activate(&mut self) -> Result<(), ShellError> {
        match self.state {
            SessionState::Starting => {
                self.state = SessionState::Active;
                Ok(())
            }
            _ => Err(ShellError::InvalidLifecycleTransition(
                "Session can only activate from the Starting state",
            )),
        }
    }

    /// Locks an active session.
    pub fn lock(&mut self) -> Result<(), ShellError> {
        match self.state {
            SessionState::Active => {
                self.state = SessionState::Locked;
                Ok(())
            }
            _ => Err(ShellError::InvalidLifecycleTransition(
                "Session can only lock from the Active state",
            )),
        }
    }

    /// Unlocks a locked session.
    pub fn unlock(&mut self) -> Result<(), ShellError> {
        match self.state {
            SessionState::Locked => {
                self.state = SessionState::Active;
                Ok(())
            }
            _ => Err(ShellError::InvalidLifecycleTransition(
                "Session can only unlock from the Locked state",
            )),
        }
    }

    /// Begins session shutdown.
    pub fn stop(&mut self) -> Result<(), ShellError> {
        match self.state {
            SessionState::Active | SessionState::Locked => {
                self.state = SessionState::Stopping;
                Ok(())
            }
            _ => Err(ShellError::InvalidLifecycleTransition(
                "Session can only stop from the Active or Locked state",
            )),
        }
    }

    /// Marks the session as fully stopped.
    pub fn finish_stop(&mut self) -> Result<(), ShellError> {
        match self.state {
            SessionState::Stopping => {
                self.state = SessionState::Stopped;
                Ok(())
            }
            _ => Err(ShellError::InvalidLifecycleTransition(
                "Session can only finish stopping from the Stopping state",
            )),
        }
    }
}

impl Default for DesktopSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "../tests/unit/session.rs"]
mod tests;
