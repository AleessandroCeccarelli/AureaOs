// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Desktop composition root.

mod compositor_factory;
mod diagnostics;
mod welcome;

use std::ffi::OsStr;

use aurea_shell::{app, config::ShellConfig};

fn main() {
    if std::env::args_os().nth(1).as_deref() == Some(OsStr::new("--welcome-client")) {
        if let Err(error) = welcome::run() {
            eprintln!("Aurea Welcome: {error}");
            std::process::exit(1);
        }
        return;
    }

    match diagnostics::initialize() {
        Ok(path) => diagnostics::record(
            "INFO",
            &format!("Starting native desktop; log: {}", path.display()),
        ),
        Err(error) => eprintln!("Aurea Desktop: persistent logging unavailable: {error}"),
    }

    diagnostics::record("INFO", "Initializing shell and DRM/Vulkan compositor");
    let config = ShellConfig::default();
    let compositor_factory = compositor_factory::DesktopCompositorFactory::default();

    if let Err(error) = app::run(config, compositor_factory) {
        diagnostics::record("ERROR", &format!("Desktop stopped with an error: {error}"));
        eprintln!("Aurea Desktop error: {error}");
        std::process::exit(1);
    }

    diagnostics::record("INFO", "Desktop stopped cleanly");
}
