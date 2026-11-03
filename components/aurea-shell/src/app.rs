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

    runtime.start()?;

    println!("Starting {}...", config.name);

    event_bus
        .publish(ShellEvent::ShutdownRequested)
        .map_err(|_| ShellError::InitializationFailed("Failed to send shutdown event"))?;

    runtime.run_event_loop(receiver)?;

    Ok(())
}
