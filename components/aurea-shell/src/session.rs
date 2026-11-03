// SPDX-License-Identifier: GPL-3.0-later
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
mod tests {
    use super::{DesktopSession, SessionState};
    use crate::error::ShellError;

    #[test]
    fn session_can_activate_lock_unlock_and_stop() {
        let mut session = DesktopSession::new();

        assert_eq!(session.state(), SessionState::Starting);

        session.activate().unwrap();
        assert_eq!(session.state(), SessionState::Active);

        session.lock().unwrap();
        assert_eq!(session.state(), SessionState::Locked);

        session.unlock().unwrap();
        assert_eq!(session.state(), SessionState::Active);

        session.stop().unwrap();
        assert_eq!(session.state(), SessionState::Stopping);

        session.finish_stop().unwrap();
        assert_eq!(session.state(), SessionState::Stopped);
    }

    #[test]
    fn session_rejects_locking_before_activation() {
        let mut session = DesktopSession::new();

        assert_eq!(
            session.lock(),
            Err(ShellError::InvalidLifecycleTransition(
                "Session can only lock from the Active state"
            ))
        );
    }

    #[test]
    fn session_rejects_unlocking_when_not_locked() {
        let mut session = DesktopSession::new();
        session.activate().unwrap();

        assert_eq!(
            session.unlock(),
            Err(ShellError::InvalidLifecycleTransition(
                "Session can only unlock from the Locked state"
            ))
        );
    }

    #[test]
    fn session_rejects_stopping_before_activation() {
        let mut session = DesktopSession::new();

        assert_eq!(
            session.stop(),
            Err(ShellError::InvalidLifecycleTransition(
                "Session can only stop from the Active or Locked state"
            ))
        );
    }

    #[test]
    fn session_rejects_finishing_stop_before_stopping() {
        let mut session = DesktopSession::new();

        assert_eq!(
            session.finish_stop(),
            Err(ShellError::InvalidLifecycleTransition(
                "Session can only finish stopping from the Stopping state"
            ))
        );
    }
}