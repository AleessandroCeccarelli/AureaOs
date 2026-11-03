// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Application error types.

use std::fmt;
use std::io;

/// Errors that can occur while running Aurea Shell.
#[derive(Debug)]
pub enum ShellError {
    InvalidLifecycleTransition(&'static str),
    InitializationFailed(&'static str),
    Compositor(std::io::Error),
    CompositorThreadPanicked,
}

impl fmt::Display for ShellError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLifecycleTransition(message) => {
                write!(formatter, "Invalid lifecycle transition: {message}")
            }
            Self::InitializationFailed(message) => {
                write!(formatter, "Shell initialization failed: {message}")
            }
            Self::Compositor(error) => {
                write!(formatter, "Wayland compositor error: {error}")
            }
            Self::CompositorThreadPanicked => {
                write!(formatter, "Wayland compositor thread panicked")
            }
        }
    }
}

impl std::error::Error for ShellError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Compositor(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for ShellError {
    fn from(error: io::Error) -> Self {
        Self::Compositor(error)
    }
}

#[cfg(test)]
mod tests {
    use super::ShellError;

    #[test]
    fn lifecycle_error_has_expected_display_message() {
        let error = ShellError::InvalidLifecycleTransition("cannot start twice");

        assert_eq!(
            error.to_string(),
            "Invalid lifecycle transition: cannot start twice"
        );
    }

    #[test]
    fn initialization_error_has_expected_display_message() {
        let error = ShellError::InitializationFailed("configuration unavailable");

        assert_eq!(
            error.to_string(),
            "Shell initialization failed: configuration unavailable"
        );
    }

    #[test]
    fn compositor_error_preserves_io_error() {
        let error = ShellError::Compositor(std::io::Error::other("socket failed"));

        assert_eq!(
            error.to_string(),
            "Wayland compositor error: socket failed"
        );
        assert!(std::error::Error::source(&error).is_some());
    }
}
