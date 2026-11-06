// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Application entry point for the AureaOS desktop shell.

use aurea_shell::{app, config::ShellConfig};

fn main() {
    let config = ShellConfig::default();

    if let Err(error) = app::run(config) {
        eprintln!("Aurea Shell error: {error}");
        std::process::exit(1);
    }
}
