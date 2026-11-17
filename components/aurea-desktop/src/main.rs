// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Desktop entry point.

mod welcome;

fn main() {
    if let Err(error) = welcome::run() {
        eprintln!("Aurea Welcome: {error}");
        std::process::exit(1);
    }
}
