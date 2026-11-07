// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Application startup and lifecycle entry point.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread::{self, JoinHandle};

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

/// Joins the compositor thread and translates its result into a shell error.
fn join_server_thread(server_thread: JoinHandle<std::io::Result<()>>) -> Result<(), ShellError> {
    match server_thread.join() {
        Ok(result) => result.map_err(ShellError::Compositor),
        Err(_) => Err(ShellError::CompositorThreadPanicked),
    }
}

/// Starts the Aurea Shell application.
pub fn run(config: ShellConfig) -> Result<(), ShellError> {
    let mut runtime = ShellRuntime::new();
    let (event_bus, receiver) = EventBus::new();

    let shutdown = Arc::new(AtomicBool::new(false));
    let server_shutdown = Arc::clone(&shutdown);
    let server_bus = event_bus.clone();

    let (ready_sender, ready_receiver) = mpsc::sync_channel(0);
    let (start_sender, start_receiver) = mpsc::sync_channel(0);

    let server_thread = thread::spawn(move || {
        // Calloop's event loop is thread-bound, so construct it in the thread
        // that will dispatch it.
        //
        // Initialize a native libseat session and open the primary DRM device
        // associated with the default Linux seat.
        let mut server = match WaylandServer::new_with_primary_drm("wayland-0", "seat0") {
            Ok(server) => server,
            Err(error) => return Err(error),
        };

        // Signal readiness only after native compositor initialization.
        ready_sender
            .send(())
            .map_err(|_| std::io::Error::other("Shell startup was cancelled"))?;

        // Keep the server initialized, but do not dispatch clients until the
        // shell lifecycle and shutdown handler are ready.
        if start_receiver.recv().is_err() {
            return Ok(());
        }

        let result = server.run(&server_shutdown);

        if result.is_err() {
            request_shutdown(&server_bus);
        }

        result
    });

    // Wait for native compositor initialization.
    if ready_receiver.recv().is_err() {
        return match server_thread.join() {
            Ok(Err(error)) => Err(ShellError::Compositor(error)),
            Ok(Ok(())) => Err(ShellError::InitializationFailed(
                "Wayland server stopped before startup",
            )),
            Err(_) => Err(ShellError::CompositorThreadPanicked),
        };
    }

    if let Err(error) = runtime.start() {
        drop(start_sender);
        let _ = server_thread.join();
        return Err(error);
    }

    println!("Starting {}...", config.name);

    let shutdown_bus = event_bus.clone();

    if ctrlc::set_handler(move || {
        request_shutdown(&shutdown_bus);
    })
    .is_err()
    {
        let runtime_result = runtime.stop();
        drop(start_sender);
        let server_result = join_server_thread(server_thread);

        // Complete both cleanups before returning either error.
        runtime_result?;
        server_result?;

        return Err(ShellError::InitializationFailed(
            "Failed to register shutdown handler",
        ));
    }

    if start_sender.send(()).is_err() {
        // The compositor exited after reporting readiness. Stop the shell
        // runtime and join the thread before reporting the startup failure.
        let runtime_result = runtime.stop();
        shutdown.store(true, Ordering::Release);
        drop(start_sender);
        let server_result = join_server_thread(server_thread);

        runtime_result?;
        server_result?;
        return Err(ShellError::InitializationFailed("Wayland server stopped"));
    }

    let runtime_result = runtime.run_event_loop(receiver);

    // Signal the compositor to exit regardless of runtime success.
    shutdown.store(true, Ordering::Release);

    let server_result = join_server_thread(server_thread);

    // Preserve the runtime error if it failed; otherwise report compositor
    // termination errors.
    runtime_result?;
    server_result?;

    Ok(())
}

#[cfg(test)]
#[path = "../tests/unit/app.rs"]
mod tests;
