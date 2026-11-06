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
    let server = WaylandServer::new("wayland-0")?;

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
        let mut server = server;
        let result = server.run(&server_shutdown);

        if result.is_err() {
            request_shutdown(&server_bus);
        }

        result
    });

    let runtime_result = runtime.run_event_loop(receiver);

    // Signal the compositor to exit regardless of runtime success.
    shutdown.store(true, Ordering::Release);

    let server_result = match server_thread.join() {
        Ok(result) => result.map_err(ShellError::Compositor),
        Err(_) => Err(ShellError::CompositorThreadPanicked),
    };

    // Preserve the runtime error if it failed; otherwise report compositor
    // termination errors.
    runtime_result?;
    server_result?;

    Ok(())
}


#[cfg(test)]
#[path = "../tests/unit/app.rs"]
mod tests;
