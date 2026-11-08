// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Application startup and lifecycle entry point.

use std::panic::{self, AssertUnwindSafe};
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

/// Starts the Aurea Shell application using the system Ctrl-C handler.
pub fn run(
    config: ShellConfig,
    compositor_factory: impl CompositorFactory,
) -> Result<(), ShellError> {
    run_with_shutdown_handler(config, compositor_factory, |handler| {
        ctrlc::set_handler(handler).is_ok()
    })
}

/// Runs the application with an injectable shutdown handler registration.
///
/// Keeping handler registration injectable makes the application lifecycle
/// testable without installing a process-global Ctrl-C handler.
fn run_with_shutdown_handler(
    config: ShellConfig,
    compositor_factory: impl CompositorFactory,
    register_shutdown_handler: impl FnOnce(Box<dyn FnMut() + Send + 'static>) -> bool,
) -> Result<(), ShellError> {
    let mut runtime = ShellRuntime::new();
    let (event_bus, receiver) = EventBus::new();

    let shutdown = ShutdownSignal::new();
    let server_shutdown = shutdown.clone();
    let server_bus = event_bus.clone();

    let (ready_sender, ready_receiver) = mpsc::sync_channel(0);
    let (start_sender, start_receiver) = mpsc::sync_channel(0);

    let compositor_thread = thread::spawn(move || {
        // Create the compositor inside its owning thread.
        let mut compositor = compositor_factory.create()?;

        // Report readiness only after successful initialization.
        ready_sender
            .send(())
            .map_err(|_| std::io::Error::other("Shell startup was cancelled"))?;

        // Wait until the shell runtime and shutdown handler are ready.
        if start_receiver.recv().is_err() {
            return Ok(());
        }

        // Convert a compositor panic into a shutdown event before
        // allowing the panic to propagate to the joining thread.
        let result = panic::catch_unwind(AssertUnwindSafe(|| compositor.run(&server_shutdown)));

        // Every compositor termination must release the shell runtime.
        request_shutdown(&server_bus);

        match result {
            Ok(result) => result,
            Err(payload) => panic::resume_unwind(payload),
        }
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

    let handler = Box::new(move || {
        request_shutdown(&shutdown_bus);
    });

    if !register_shutdown_handler(handler) {
        let runtime_result = runtime.stop();

        drop(start_sender);

        let compositor_result = join_compositor_thread(compositor_thread);

        runtime_result?;
        compositor_result?;

        return Err(ShellError::InitializationFailed(
            "Failed to register shutdown handler",
        ));
    }

    if start_sender.send(()).is_err() {
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

    // Always request compositor shutdown before joining its thread.
    shutdown.request_shutdown();

    let compositor_result = join_compositor_thread(compositor_thread);

    runtime_result?;
    compositor_result?;

    Ok(())
}

#[cfg(test)]
#[path = "../tests/unit/app.rs"]
mod tests;
