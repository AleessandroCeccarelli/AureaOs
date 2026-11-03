// SPDX-License-Identifier: GPL-3.0-only
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Application error types.

use std::fmt;

/// Errors that can occur while running Aurea Shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellError {
    InvalidLifecycleTransition(&'static str),
    InitializationFailed(&'static str),
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
        }
    }
}

impl std::error::Error for ShellError {}

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
}
