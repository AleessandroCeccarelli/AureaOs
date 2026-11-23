// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS test module relocated from production source.

use super::{RuntimeCommand, dispatch};
use crate::event::ShellEvent;

#[test]
fn dispatches_lock_event() {
    assert_eq!(
        dispatch(ShellEvent::LockRequested),
        RuntimeCommand::LockSession
    );
}

#[test]
fn dispatches_unlock_event() {
    assert_eq!(
        dispatch(ShellEvent::UnlockRequested),
        RuntimeCommand::UnlockSession
    );
}

#[test]
fn dispatches_shutdown_event() {
    assert_eq!(
        dispatch(ShellEvent::ShutdownRequested),
        RuntimeCommand::Shutdown
    );
}
