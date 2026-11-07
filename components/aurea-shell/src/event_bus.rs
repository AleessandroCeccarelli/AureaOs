// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Internal shell event channel.

use std::sync::mpsc::{self, Receiver, Sender};

use crate::event::ShellEvent;

/// Sends events to the shell runtime.
#[derive(Clone)]
pub struct EventBus {
    sender: Sender<ShellEvent>,
}

impl EventBus {
    /// Creates an event bus and its receiving endpoint.
    pub fn new() -> (Self, Receiver<ShellEvent>) {
        let (sender, receiver) = mpsc::channel();

        (Self { sender }, receiver)
    }

    /// Publishes an event to the runtime.
    pub fn publish(&self, event: ShellEvent) -> Result<(), ShellEvent> {
        self.sender.send(event).map_err(|error| error.0)
    }
}

#[cfg(test)]
#[path = "../tests/unit/event_bus.rs"]
mod tests;
