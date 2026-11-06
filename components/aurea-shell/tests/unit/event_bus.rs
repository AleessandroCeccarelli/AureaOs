// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS test module relocated from production source.

use super::EventBus;
use crate::event::ShellEvent;

#[test]
fn event_bus_publishes_and_receiver_gets_event() {
    let (event_bus, receiver) = EventBus::new();

    event_bus.publish(ShellEvent::LockRequested).unwrap();

    assert_eq!(receiver.recv().unwrap(), ShellEvent::LockRequested);
}

#[test]
fn event_bus_publishes_multiple_events_in_order() {
    let (event_bus, receiver) = EventBus::new();

    event_bus.publish(ShellEvent::LockRequested).unwrap();
    event_bus.publish(ShellEvent::UnlockRequested).unwrap();
    event_bus.publish(ShellEvent::ShutdownRequested).unwrap();

    assert_eq!(receiver.recv().unwrap(), ShellEvent::LockRequested);
    assert_eq!(receiver.recv().unwrap(), ShellEvent::UnlockRequested);
    assert_eq!(receiver.recv().unwrap(), ShellEvent::ShutdownRequested);
}

#[test]
fn cloned_event_bus_can_publish_to_same_receiver() {
    let (event_bus, receiver) = EventBus::new();
    let cloned_bus = event_bus.clone();

    event_bus.publish(ShellEvent::LockRequested).unwrap();
    cloned_bus.publish(ShellEvent::UnlockRequested).unwrap();

    assert_eq!(receiver.recv().unwrap(), ShellEvent::LockRequested);
    assert_eq!(receiver.recv().unwrap(), ShellEvent::UnlockRequested);
}

#[test]
fn event_bus_returns_event_when_receiver_is_dropped() {
    let (event_bus, receiver) = EventBus::new();
    drop(receiver);

    let result = event_bus.publish(ShellEvent::ShutdownRequested);

    assert_eq!(result, Err(ShellEvent::ShutdownRequested));
}
