// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Shell runtime management.

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
    use super::ShellRuntime;
    use crate::event::ShellEvent;
    use crate::state::ShellState;

    #[test]
    fn runtime_starts_and_stops_on_shutdown_event() {
        let mut runtime = ShellRuntime::new();

        assert_eq!(runtime.state(), ShellState::Created);

        runtime.start().unwrap();
        assert_eq!(runtime.state(), ShellState::Running);

        runtime
            .handle_event(ShellEvent::ShutdownRequested)
            .unwrap();

        assert_eq!(runtime.state(), ShellState::Stopped);
    }
}
