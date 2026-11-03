// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 AureaOS project contributors
//
// Application entry point for the AureaOS desktop shell.

mod config;

fn main() {
    let config = config::ShellConfig::default();

    println!("Starting {}...", config.name);
}