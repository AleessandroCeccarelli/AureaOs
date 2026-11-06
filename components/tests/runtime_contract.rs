// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Runtime integration contract tests.

use crate::error::ShellError;
use crate::event::ShellEvent;
use crate::event_bus::EventBus;
use crate::runtime::ShellRuntime;
use crate::session::SessionState;
use crate::state::ShellState;

#[test]
fn runtime_shutdown_completes_both_lifecycles() {
    let mut runtime = ShellRuntime::new();

    runtime.start().unwrap();
    runtime.handle_event(ShellEvent::ShutdownRequested).unwrap();

    assert_eq!(runtime.state(), ShellState::Stopped);
    assert_eq!(runtime.session_state(), SessionState::Stopped);
}

#[test]
fn runtime_preserves_state_after_invalid_unlock() {
    let mut runtime = ShellRuntime::new();

    runtime.start().unwrap();

    let result = runtime.handle_event(ShellEvent::UnlockRequested);

    assert!(matches!(
        result,
        Err(ShellError::InvalidLifecycleTransition(_))
    ));

    assert_eq!(runtime.state(), ShellState::Running);
    assert_eq!(runtime.session_state(), SessionState::Active);
}

#[test]
fn runtime_event_loop_handles_shutdown_from_event_bus() {
    let (event_bus, receiver) = EventBus::new();
    let mut runtime = ShellRuntime::new();

    runtime.start().unwrap();

    event_bus.publish(ShellEvent::LockRequested).unwrap();
    event_bus.publish(ShellEvent::UnlockRequested).unwrap();
    event_bus.publish(ShellEvent::ShutdownRequested).unwrap();

    runtime.run_event_loop(receiver).unwrap();

    assert_eq!(runtime.state(), ShellState::Stopped);
    assert_eq!(runtime.session_state(), SessionState::Stopped);
}
