// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Application startup and lifecycle entry point.

use crate::config::ShellConfig;

/// Starts the Aurea Shell application.
pub fn run(config: ShellConfig) {
    println!("Starting {}...", config.name);
}