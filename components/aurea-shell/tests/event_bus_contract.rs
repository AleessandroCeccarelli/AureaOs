// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS shell integration contracts.

use aurea_shell::event::ShellEvent;
use aurea_shell::event_bus::EventBus;

#[test]
fn cloned_event_bus_shares_the_same_channel() {
    let (bus, receiver) = EventBus::new();
    let second_bus = bus.clone();

    bus.publish(ShellEvent::LockRequested).unwrap();
    second_bus.publish(ShellEvent::ShutdownRequested).unwrap();

    assert_eq!(receiver.recv().unwrap(), ShellEvent::LockRequested);

    assert_eq!(receiver.recv().unwrap(), ShellEvent::ShutdownRequested);
}

#[test]
fn publishing_to_disconnected_receiver_returns_original_event() {
    let (bus, receiver) = EventBus::new();

    drop(receiver);

    assert_eq!(
        bus.publish(ShellEvent::ShutdownRequested),
        Err(ShellEvent::ShutdownRequested)
    );
}
