
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Shell runtime management.

use crate::error::ShellError;
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
    use crate::state::ShellState;

    #[test]
    fn runtime_starts_and_stops() {
        let mut runtime = ShellRuntime::new();

        assert_eq!(runtime.state(), ShellState::Created);

        runtime.start().unwrap();
        assert_eq!(runtime.state(), ShellState::Running);

        runtime.stop().unwrap();
        assert_eq!(runtime.state(), ShellState::Stopped);
    }
}
