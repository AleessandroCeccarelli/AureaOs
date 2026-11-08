// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS shell integration contracts.

use aurea_shell::error::ShellError;
use aurea_shell::event::ShellEvent;
use aurea_shell::event_bus::EventBus;
use aurea_shell::runtime::ShellRuntime;
use aurea_shell::session::SessionState;
use aurea_shell::state::ShellState;

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

#[test]
fn runtime_stops_when_event_channel_is_disconnected() {
    let (event_bus, receiver) = EventBus::new();
    let mut runtime = ShellRuntime::new();

    runtime.start().unwrap();

    drop(event_bus);

    runtime.run_event_loop(receiver).unwrap();

    assert_eq!(runtime.state(), ShellState::Stopped);
    assert_eq!(runtime.session_state(), SessionState::Stopped);
}
