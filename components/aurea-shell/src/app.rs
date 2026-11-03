// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Application startup and lifecycle entry point.

use std::sync::mpsc;

use crate::config::ShellConfig;
use crate::error::ShellError;
use crate::event::ShellEvent;
use crate::runtime::ShellRuntime;

/// Starts the Aurea Shell application.
pub fn run(config: ShellConfig) -> Result<(), ShellError> {
    let mut runtime = ShellRuntime::new();
    let (sender, receiver) = mpsc::channel();

    runtime.start()?;

    println!("Starting {}...", config.name);

    sender
        .send(ShellEvent::ShutdownRequested)
        .map_err(|_| ShellError::InitializationFailed("Failed to send shutdown event"))?;

    runtime.run_event_loop(receiver)?;

    Ok(())
}
