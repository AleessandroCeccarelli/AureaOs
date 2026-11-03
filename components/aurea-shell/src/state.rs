// SPDX-License-Identifier: GPL-3.0-only
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
mod tests {
    use super::{ShellLifecycle, ShellState};
    use crate::error::ShellError;

    #[test]
    fn lifecycle_moves_from_created_to_running_to_stopped() {
        let mut lifecycle = ShellLifecycle::new();

        assert_eq!(lifecycle.state(), ShellState::Created);

        lifecycle.start().unwrap();
        assert_eq!(lifecycle.state(), ShellState::Running);

        lifecycle.stop().unwrap();
        assert_eq!(lifecycle.state(), ShellState::Stopped);
    }

    #[test]
    fn lifecycle_rejects_starting_twice() {
        let mut lifecycle = ShellLifecycle::new();
        lifecycle.start().unwrap();

        assert_eq!(
            lifecycle.start(),
            Err(ShellError::InvalidLifecycleTransition(
                "Shell can only start from the Created state"
            ))
        );
    }

    #[test]
    fn lifecycle_rejects_stopping_before_start() {
        let mut lifecycle = ShellLifecycle::new();

        assert_eq!(
            lifecycle.stop(),
            Err(ShellError::InvalidLifecycleTransition(
                "Shell can only stop from the Running state"
            ))
        );
    }
}
