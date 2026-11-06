// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Application lifecycle state.

use crate::error::ShellError;

/// Current lifecycle state of the shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellState {
    Created,
    Running,
    Stopped,
}

/// Tracks and validates lifecycle transitions.
pub struct ShellLifecycle {
    state: ShellState,
}

impl ShellLifecycle {
    /// Creates a lifecycle tracker in the initial state.
    pub fn new() -> Self {
        Self {
            state: ShellState::Created,
        }
    }

    /// Returns the current lifecycle state.
    pub fn state(&self) -> ShellState {
        self.state
    }

    /// Starts the shell if it has not already been started.
    pub fn start(&mut self) -> Result<(), ShellError> {
        match self.state {
            ShellState::Created => {
                self.state = ShellState::Running;
                Ok(())
            }
            _ => Err(ShellError::InvalidLifecycleTransition(
                "Shell can only start from the Created state",
            )),
        }
    }

    /// Stops the shell if it is currently running.
    pub fn stop(&mut self) -> Result<(), ShellError> {
        match self.state {
            ShellState::Running => {
                self.state = ShellState::Stopped;
                Ok(())
            }
            _ => Err(ShellError::InvalidLifecycleTransition(
                "Shell can only stop from the Running state",
            )),
        }
    }
}

impl Default for ShellLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "../tests/unit/state.rs"]
mod tests;
