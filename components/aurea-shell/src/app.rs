// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Application startup and lifecycle entry point.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use aurea_compositor::WaylandServer;

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

    // Bind the Wayland socket before starting the shell lifecycle.
    let server = WaylandServer::new("wayland-0")
        .map_err(|_| ShellError::InitializationFailed(
            "Failed to initialize Wayland compositor",
        ))?;

    runtime.start()?;

    println!("Starting {}...", config.name);

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_bus = event_bus.clone();

    if ctrlc::set_handler(move || {
        request_shutdown(&shutdown_bus);
    })
    .is_err()
    {
        runtime.stop()?;

        return Err(ShellError::InitializationFailed(
            "Failed to register shutdown handler",
        ));
    }

    let server_shutdown = Arc::clone(&shutdown);
    let server_bus = event_bus.clone();

    let server_thread = thread::spawn(move || {
        let result = {
            let mut server = server;
            server.run(&server_shutdown)
        };

        if result.is_err() {
            request_shutdown(&server_bus);
        }

        result
    });

    let runtime_result = runtime.run_event_loop(receiver);

    shutdown.store(true, Ordering::Relaxed);

    let server_result = server_thread.join();

    runtime_result?;

    match server_result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(_)) => Err(ShellError::InitializationFailed(
            "Wayland compositor stopped unexpectedly",
        )),
        Err(_) => Err(ShellError::InitializationFailed(
            "Wayland compositor thread panicked",
        )),
    }
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
