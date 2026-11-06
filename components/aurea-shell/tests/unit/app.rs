// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS test module relocated from production source.

use super::request_shutdown;
use crate::event::ShellEvent;
use crate::event_bus::EventBus;

#[test]
fn shutdown_request_publishes_shutdown_event() {
    let (event_bus, receiver) = EventBus::new();

    request_shutdown(&event_bus);

    assert_eq!(receiver.recv().unwrap(), ShellEvent::ShutdownRequested);
}
