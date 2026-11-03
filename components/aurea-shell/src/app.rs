
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Application startup and lifecycle entry point.

use crate::config::ShellConfig;
use crate::error::ShellError;
use crate::runtime::ShellRuntime;

/// Starts the Aurea Shell application.
pub fn run(config: ShellConfig) -> Result<(), ShellError> {
    let mut runtime = ShellRuntime::new();

    runtime.start()?;

    println!("Starting {}...", config.name);

    runtime.stop()?;

    Ok(())
}
