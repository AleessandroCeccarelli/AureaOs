// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Application startup and lifecycle entry point.

use crate::config::ShellConfig;
use crate::state::ShellLifecycle;

/// Starts the Aurea Shell application.
pub fn run(config: ShellConfig) {
    let mut lifecycle = ShellLifecycle::new();

    if let Err(error) = lifecycle.start() {
        eprintln!("{error}");
        return;
    }

    println!("Starting {}...", config.name);
}
