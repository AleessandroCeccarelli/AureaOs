
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Desktop application entry point.

use aurea_compositor::service::WaylandCompositorFactory;
use aurea_shell::{app, config::ShellConfig};

fn main() {
    let config = ShellConfig::default();
    let compositor_factory = WaylandCompositorFactory::default();

    if let Err(error) = app::run(config, compositor_factory) {
        eprintln!("Aurea Desktop error: {error}");
        std::process::exit(1);
    }
}
