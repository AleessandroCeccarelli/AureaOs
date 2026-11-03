// SPDX-License-Identifier: GPL-3.0-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Application startup and lifecycle entry point.

use crate::config::ShellConfig;
use crate::error::ShellError;
use crate::event::ShellEvent;
use crate::event_bus::EventBus;
use crate::runtime::ShellRuntime;

/// Requests runtime shutdown through the event bus.
fn request_shutdown(event_bus: &EventBus) {
    let _ = event_bus.publish(ShellEvent::ShutdownRequested);
}

/// Starts the Aurea Shell application.
pub fn run(config: ShellConfig) -> Result<(), ShellError> {
    let mut runtime = ShellRuntime::new();
    let (event_bus, receiver) = EventBus::new();

    runtime.start()?;

    println!("Starting {}...", config.name);

    let shutdown_bus = event_bus.clone();

    ctrlc::set_handler(move || {
        request_shutdown(&shutdown_bus);
    })
    .map_err(|_| ShellError::InitializationFailed("Failed to register shutdown handler"))?;

    runtime.run_event_loop(receiver)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::request_shutdown;
    use crate::event::ShellEvent;
    use crate::event_bus::EventBus;

    #[test]
    fn shutdown_request_publishes_shutdown_event() {
        let (event_bus, receiver) = EventBus::new();

        request_shutdown(&event_bus);

        assert_eq!(
            receiver.recv().unwrap(),
            ShellEvent::ShutdownRequested
        );
    }
}
