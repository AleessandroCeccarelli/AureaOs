// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS test module relocated from production source.

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

    assert!(matches!(
        lifecycle.start(),
        Err(ShellError::InvalidLifecycleTransition(
            "Shell can only start from the Created state"
        ))
    ));
}

#[test]
fn lifecycle_rejects_stopping_before_start() {
    let mut lifecycle = ShellLifecycle::new();

    assert!(matches!(
        lifecycle.stop(),
        Err(ShellError::InvalidLifecycleTransition(
            "Shell can only stop from the Running state"
        ))
    ));
}
