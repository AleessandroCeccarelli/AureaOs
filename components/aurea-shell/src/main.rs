
 // SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Application entry point for the AureaOS desktop shell.

mod app;
mod config;
mod error;
mod state;

fn main() {
    let config = config::ShellConfig::default();

    if let Err(error) = app::run(config) {
        eprintln!("Aurea Shell error: {error}");
    }
}
