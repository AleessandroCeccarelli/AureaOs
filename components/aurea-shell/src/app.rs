// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Application startup and lifecycle entry point.

use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use aurea_desktop_api::{CompositorFactory, ShutdownSignal};

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
fn join_compositor_thread(
    compositor_thread: JoinHandle<std::io::Result<()>>,
) -> Result<(), ShellError> {
    match compositor_thread.join() {
        Ok(result) => result.map_err(ShellError::Compositor),
        Err(_) => Err(ShellError::CompositorThreadPanicked),
    }
}

/// Starts the Aurea Shell application using the supplied compositor factory.
pub fn run(
    config: ShellConfig,
    compositor_factory: impl CompositorFactory,
) -> Result<(), ShellError> {
    let mut runtime = ShellRuntime::new();
    let (event_bus, receiver) = EventBus::new();

    let shutdown = ShutdownSignal::new();
    let server_shutdown = shutdown.clone();
    let server_bus = event_bus.clone();

    let (ready_sender, ready_receiver) = mpsc::sync_channel(0);
    let (start_sender, start_receiver) = mpsc::sync_channel(0);

    let compositor_thread = thread::spawn(move || {
        // The compositor service is created inside its owning thread.
        // This keeps thread-bound event loops such as Calloop on the
        // same thread where they are initialized and dispatched.
        let mut compositor = compositor_factory.create()?;

        // Signal readiness only after compositor initialization.
        ready_sender
            .send(())
            .map_err(|_| std::io::Error::other("Shell startup was cancelled"))?;

        // Keep the compositor initialized, but do not dispatch clients
        // until the shell lifecycle and shutdown handler are ready.
        if start_receiver.recv().is_err() {
            return Ok(());
        }

        let result = compositor.run(&server_shutdown);

        if result.is_err() {
            request_shutdown(&server_bus);
        }

        result
    });

    // Wait for compositor initialization.
    if ready_receiver.recv().is_err() {
        return match compositor_thread.join() {
            Ok(Err(error)) => Err(ShellError::Compositor(error)),
            Ok(Ok(())) => Err(ShellError::InitializationFailed(
                "Compositor stopped before startup",
            )),
            Err(_) => Err(ShellError::CompositorThreadPanicked),
        };
    }

    if let Err(error) = runtime.start() {
        drop(start_sender);
        let _ = compositor_thread.join();
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
        let compositor_result = join_compositor_thread(compositor_thread);

        // Complete both cleanups before returning either error.
        runtime_result?;
        compositor_result?;

        return Err(ShellError::InitializationFailed(
            "Failed to register shutdown handler",
        ));
    }

    if start_sender.send(()).is_err() {
        // The compositor exited after reporting readiness.
        // Stop the shell runtime and join the thread before reporting
        // the startup failure.
        let runtime_result = runtime.stop();
        shutdown.request_shutdown();
        drop(start_sender);
        let compositor_result = join_compositor_thread(compositor_thread);

        runtime_result?;
        compositor_result?;

        return Err(ShellError::InitializationFailed(
            "Compositor stopped during startup",
        ));
    }

    let runtime_result = runtime.run_event_loop(receiver);

    // Signal the compositor to exit regardless of runtime success.
    shutdown.request_shutdown();

    let compositor_result = join_compositor_thread(compositor_thread);

    // Preserve the runtime error if it failed; otherwise report
    // compositor termination errors.
    runtime_result?;
    compositor_result?;

    Ok(())
}

#[cfg(test)]
#[path = "../tests/unit/app.rs"]
mod tests;
