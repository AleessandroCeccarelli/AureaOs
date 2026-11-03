// SPDX-License-Identifier: GPL-3.0-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Shell runtime management.

use std::sync::mpsc::Receiver;

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
    pub fn start(&mut self) -> Result<(), ShellError> {
        self.lifecycle.start()?;
        self.session.activate()
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

    /// Handles an event received by the runtime.
    pub fn handle_event(&mut self, event: ShellEvent) -> Result<(), ShellError> {
        match event {
            ShellEvent::ShutdownRequested => self.stop(),
        }
    }

    /// Stops the desktop session and then the runtime.
    pub fn stop(&mut self) -> Result<(), ShellError> {
        if self.session.state() == SessionState::Active
            || self.session.state() == SessionState::Locked
        {
            self.session.stop()?;
        }

        if self.session.state() == SessionState::Stopping {
            self.session.finish_stop()?;
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
mod tests {
    use std::sync::mpsc;

    use super::ShellRuntime;
    use crate::event::ShellEvent;
    use crate::session::SessionState;
    use crate::state::ShellState;

    #[test]
    fn runtime_event_loop_stops_on_shutdown_event() {
        let (sender, receiver) = mpsc::channel();
        let mut runtime = ShellRuntime::new();

        runtime.start().unwrap();
        sender.send(ShellEvent::ShutdownRequested).unwrap();

        runtime.run_event_loop(receiver).unwrap();

        assert_eq!(runtime.state(), ShellState::Stopped);
        assert_eq!(runtime.session_state(), SessionState::Stopped);
    }

    #[test]
    fn runtime_event_loop_stops_when_channel_closes() {
        let (sender, receiver) = mpsc::channel();
        let mut runtime = ShellRuntime::new();

        runtime.start().unwrap();
        drop(sender);

        runtime.run_event_loop(receiver).unwrap();

        assert_eq!(runtime.state(), ShellState::Stopped);
        assert_eq!(runtime.session_state(), SessionState::Stopped);
    }
}
