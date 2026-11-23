// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS test module relocated from production source.

use super::{DesktopSession, SessionState};
use crate::error::ShellError;

#[test]
fn session_can_activate_lock_unlock_and_stop() {
    let mut session = DesktopSession::new();

    assert_eq!(session.state(), SessionState::Starting);

    session.activate().unwrap();
    assert_eq!(session.state(), SessionState::Active);

    session.lock().unwrap();
    assert_eq!(session.state(), SessionState::Locked);

    session.unlock().unwrap();
    assert_eq!(session.state(), SessionState::Active);

    session.stop().unwrap();
    assert_eq!(session.state(), SessionState::Stopping);

    session.finish_stop().unwrap();
    assert_eq!(session.state(), SessionState::Stopped);
}

#[test]
fn session_rejects_locking_before_activation() {
    let mut session = DesktopSession::new();

    assert!(matches!(
        session.lock(),
        Err(ShellError::InvalidLifecycleTransition(
            "Session can only lock from the Active state"
        ))
    ));
}

#[test]
fn session_rejects_unlocking_when_not_locked() {
    let mut session = DesktopSession::new();
    session.activate().unwrap();

    assert!(matches!(
        session.unlock(),
        Err(ShellError::InvalidLifecycleTransition(
            "Session can only unlock from the Locked state"
        ))
    ));
}

#[test]
fn session_rejects_stopping_before_activation() {
    let mut session = DesktopSession::new();

    assert!(matches!(
        session.stop(),
        Err(ShellError::InvalidLifecycleTransition(
            "Session can only stop from the Active or Locked state"
        ))
    ));
}

#[test]
fn session_rejects_finishing_stop_before_stopping() {
    let mut session = DesktopSession::new();

    assert!(matches!(
        session.finish_stop(),
        Err(ShellError::InvalidLifecycleTransition(
            "Session can only finish stopping from the Stopping state"
        ))
    ));
}
