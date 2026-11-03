
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Application startup and lifecycle entry point.

use crate::config::ShellConfig;
use crate::error::ShellError;
use crate::state::ShellLifecycle;

/// Starts the Aurea Shell application.
pub fn run(config: ShellConfig) -> Result<(), ShellError> {
    let mut lifecycle = ShellLifecycle::new();

    lifecycle.start()?;

    println!("Starting {}...", config.name);

    Ok(())
}
