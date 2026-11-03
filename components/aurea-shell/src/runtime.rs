// SPDX-License-Identifier: GPL-3.0-only
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Shell runtime management.

use std::sync::mpsc::Receiver;

use crate::error::ShellError;
use crate::event::ShellEvent;
use crate::state::{ShellLifecycle, ShellState};

/// Owns the runtime lifecycle of the shell.
pub struct ShellRuntime {
    lifecycle: ShellLifecycle,
}

impl ShellRuntime {
    /// Creates a runtime that has not been started yet.
    pub fn new() -> Self {
        Self {
            lifecycle: ShellLifecycle::new(),
        }
    }

    /// Starts the runtime.
    pub fn start(&mut self) -> Result<(), ShellError> {
        self.lifecycle.start()
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

    /// Stops the runtime.
    pub fn stop(&mut self) -> Result<(), ShellError> {
        self.lifecycle.stop()
    }

    /// Returns the current runtime state.
    pub fn state(&self) -> ShellState {
        self.lifecycle.state()
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
    use crate::state::ShellState;

    #[test]
    fn runtime_event_loop_stops_on_shutdown_event() {
        let (sender, receiver) = mpsc::channel();
        let mut runtime = ShellRuntime::new();

        runtime.start().unwrap();
        sender.send(ShellEvent::ShutdownRequested).unwrap();

        runtime.run_event_loop(receiver).unwrap();

        assert_eq!(runtime.state(), ShellState::Stopped);
    }

    #[test]
    fn runtime_event_loop_stops_when_channel_closes() {
        let (sender, receiver) = mpsc::channel();
        let mut runtime = ShellRuntime::new();

        runtime.start().unwrap();
        drop(sender);

        runtime.run_event_loop(receiver).unwrap();

        assert_eq!(runtime.state(), ShellState::Stopped);
    }
}
