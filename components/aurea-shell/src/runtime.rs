// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Shell runtime management.

use std::sync::mpsc::Receiver;

use crate::dispatcher::{self, RuntimeCommand};
use crate::error::ShellError;
use crate::event::ShellEvent;
use crate::session::{DesktopSession, SessionState};
use crate::state::{ShellLifecycle, ShellState};

/// Owns the runtime and desktop session lifecycles.
pub struct ShellRuntime {
    lifecycle: ShellLifecycle,
    session: DesktopSession,
}

impl ShellRuntime {
    /// Creates a runtime that has not been started yet.
    pub fn new() -> Self {
        Self {
            lifecycle: ShellLifecycle::new(),
            session: DesktopSession::new(),
        }
    }

    /// Starts the runtime and activates the desktop session.
    ///
    /// If session activation fails, the runtime lifecycle is rolled back.
    pub fn start(&mut self) -> Result<(), ShellError> {
        self.lifecycle.start()?;

        if let Err(error) = self.session.activate() {
            self.lifecycle.stop()?;
            return Err(error);
        }

        Ok(())
    }

    /// Runs the event loop until shutdown is requested or the channel closes.
    pub fn run_event_loop(
        &mut self,
        receiver: Receiver<ShellEvent>,
    ) -> Result<(), ShellError> {
        while self.state() == ShellState::Running {
            match receiver.recv() {
                Ok(event) => self.handle_event(event)?,
                Err(_) => self.stop()?,
            }
        }

        Ok(())
    }

    /// Dispatches an event and executes the resulting runtime command.
    pub fn handle_event(&mut self, event: ShellEvent) -> Result<(), ShellError> {
        match dispatcher::dispatch(event) {
            RuntimeCommand::LockSession => self.session.lock(),
            RuntimeCommand::UnlockSession => self.session.unlock(),
            RuntimeCommand::Shutdown => self.stop(),
        }
    }

    /// Stops the desktop session and then the runtime.
    pub fn stop(&mut self) -> Result<(), ShellError> {
        match self.session.state() {
            SessionState::Active | SessionState::Locked => {
                self.session.stop()?;
                self.session.finish_stop()?;
            }
            SessionState::Stopping => {
                self.session.finish_stop()?;
            }
            SessionState::Stopped => {}
            SessionState::Starting => {
                return Err(ShellError::InvalidLifecycleTransition(
                    "Session cannot stop before activation",
                ));
            }
        }

        self.lifecycle.stop()
    }

    /// Returns the current runtime state.
    pub fn state(&self) -> ShellState {
        self.lifecycle.state()
    }

    /// Returns the current desktop session state.
    pub fn session_state(&self) -> SessionState {
        self.session.state()
    }
}

impl Default for ShellRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "../tests/unit/runtime.rs"]
mod tests;
