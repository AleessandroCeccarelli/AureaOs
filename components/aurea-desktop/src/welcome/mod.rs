// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! First native Wayland client used to validate visible desktop surfaces.

mod buffer;
mod dimensions;
mod frame;
mod painter;
mod protocol;
mod scaling;
mod typography;

// Shared dimensions for the Welcome surface and its SHM buffer.
pub(super) const WIDTH: usize = 720;
pub(super) const HEIGHT: usize = 440;

use std::io;

use wayland_client::{
    Connection,
    globals::registry_queue_init,
    protocol::{wl_compositor::WlCompositor, wl_shm::WlShm, wl_surface::WlSurface},
};
use wayland_protocols::xdg::shell::client::xdg_wm_base::XdgWmBase;

pub(super) struct WelcomeWindow {
    shm: WlShm,
    surface: WlSurface,
    frames: Vec<frame::FrameResources>,
    configured_size: Option<(i32, i32)>,
    current_size: Option<(usize, usize)>,
    closed: bool,
}

impl WelcomeWindow {
    fn new(shm: WlShm, surface: WlSurface) -> Self {
        Self {
            shm,
            surface,
            frames: Vec::new(),
            configured_size: None,
            current_size: None,
            closed: false,
        }
    }
}

pub fn run() -> io::Result<()> {
    let connection = Connection::connect_to_env()
        .map_err(|error| io::Error::other(format!("cannot connect to Wayland: {error}")))?;
    let (globals, mut queue) = registry_queue_init::<WelcomeWindow>(&connection)
        .map_err(|error| io::Error::other(format!("Wayland registry failed: {error}")))?;
    let qh = queue.handle();

    let compositor = globals
        .bind::<WlCompositor, _, _>(&qh, 4..=6, ())
        .map_err(|error| io::Error::other(format!("wl_compositor unavailable: {error}")))?;
    let shm = globals
        .bind::<WlShm, _, _>(&qh, 1..=1, ())
        .map_err(|error| io::Error::other(format!("wl_shm unavailable: {error}")))?;
    let wm_base = globals
        .bind::<XdgWmBase, _, _>(&qh, 1..=6, ())
        .map_err(|error| io::Error::other(format!("xdg_wm_base unavailable: {error}")))?;

    let surface = compositor.create_surface(&qh, ());
    let xdg_surface = wm_base.get_xdg_surface(&surface, &qh, ());
    let toplevel = xdg_surface.get_toplevel(&qh, ());
    toplevel.set_title("AureaOS — Welcome".to_owned());
    toplevel.set_app_id("org.aureaos.Welcome".to_owned());
    toplevel.set_min_size(560, 360);
    surface.commit();

    let mut state = WelcomeWindow::new(shm, surface);

    // Keep protocol objects alive for the complete client lifetime.
    let _xdg_surface = xdg_surface;
    let _toplevel = toplevel;
    let _wm_base = wm_base;
    let _compositor = compositor;

    while !state.closed {
        queue
            .blocking_dispatch(&mut state)
            .map_err(|error| io::Error::other(format!("Wayland dispatch failed: {error}")))?;
    }

    Ok(())
}
