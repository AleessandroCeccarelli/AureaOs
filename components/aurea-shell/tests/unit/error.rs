// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS test module relocated from production source.

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

    assert_eq!(error.to_string(), "Wayland compositor error: socket failed");
    assert!(std::error::Error::source(&error).is_some());
}
