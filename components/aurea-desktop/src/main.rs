// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Desktop composition root.

mod diagnostics;

use aurea_compositor::service::WaylandCompositorFactory;
use aurea_shell::{app, config::ShellConfig};

fn main() {
    match diagnostics::initialize() {
        Ok(path) => diagnostics::record(
            "INFO",
            &format!("Starting native desktop; log: {}", path.display()),
        ),
        Err(error) => eprintln!("Aurea Desktop: persistent logging unavailable: {error}"),
    }

    diagnostics::record("INFO", "Initializing shell and DRM/Vulkan compositor");
    let config = ShellConfig::default();
    let compositor_factory = WaylandCompositorFactory::default();

    if let Err(error) = app::run(config, compositor_factory) {
        diagnostics::record("ERROR", &format!("Desktop stopped with an error: {error}"));
        eprintln!("Aurea Desktop error: {error}");
        std::process::exit(1);
    }

    diagnostics::record("INFO", "Desktop stopped cleanly");
}
