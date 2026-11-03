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

/// Starts the Aurea Shell application.
pub fn run(config: ShellConfig) -> Result<(), ShellError> {
    let mut runtime = ShellRuntime::new();
    let (event_bus, receiver) = EventBus::new();

    let shutdown_bus = event_bus.clone();

    ctrlc::set_handler(move || {
        let _ = shutdown_bus.publish(ShellEvent::ShutdownRequested);
    })
    .map_err(|_| ShellError::InitializationFailed("Failed to register shutdown handler"))?;

    runtime.start()?;

    println!("Starting {}...", config.name);

    runtime.run_event_loop(receiver)?;

    Ok(())
}
