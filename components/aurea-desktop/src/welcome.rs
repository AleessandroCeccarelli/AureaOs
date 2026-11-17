// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! First native Wayland client used to validate visible desktop surfaces.

use std::{fs::OpenOptions, io, os::fd::AsFd, path::PathBuf};

use memmap2::MmapMut;
use wayland_client::{
    Connection, Dispatch, QueueHandle, delegate_noop,
    globals::registry_queue_init,
    protocol::{
        wl_buffer::{self, WlBuffer},
        wl_compositor::{self, WlCompositor},
        wl_registry,
        wl_shm::{self, Format, WlShm},
        wl_shm_pool::{self, WlShmPool},
        wl_surface::{self, WlSurface},
    },
};
use wayland_protocols::xdg::shell::client::{
    xdg_surface::{self, XdgSurface},
    xdg_toplevel::{self, XdgToplevel},
    xdg_wm_base::{self, XdgWmBase},
};

const WIDTH: u32 = 720;
const HEIGHT: u32 = 440;
const STRIDE: u32 = WIDTH * 4;
const BUFFER_SIZE: u32 = STRIDE * HEIGHT;

struct WelcomeWindow {
    shm: WlShm,
    surface: WlSurface,
    pool: Option<WlShmPool>,
    buffer: Option<WlBuffer>,
    pixels: Option<MmapMut>,
    closed: bool,
}

impl WelcomeWindow {
    fn new(shm: WlShm, surface: WlSurface) -> Self {
        Self {
            shm,
            surface,
            pool: None,
            buffer: None,
            pixels: None,
            closed: false,
        }
    }

    fn draw(&mut self, qh: &QueueHandle<Self>) -> io::Result<()> {
        if self.buffer.is_some() {
            return Ok(());
        }

        let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "XDG_RUNTIME_DIR is not set"))?;
        let path = runtime_dir.join(format!("aurea-welcome-{}.shm", std::process::id()));

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)?;
        let _ = std::fs::remove_file(&path);
        file.set_len(u64::from(BUFFER_SIZE))?;

        let mut pixels = unsafe { MmapMut::map_mut(&file)? };
        paint_welcome(&mut pixels);

        let pool = self
            .shm
            .create_pool(file.as_fd(), BUFFER_SIZE as i32, qh, ());
        let buffer = pool.create_buffer(
            0,
            WIDTH as i32,
            HEIGHT as i32,
            STRIDE as i32,
            Format::Argb8888,
            qh,
            (),
        );

        self.surface.attach(Some(&buffer), 0, 0);
        self.surface.damage(0, 0, WIDTH as i32, HEIGHT as i32);
        self.surface.commit();

        self.pool = Some(pool);
        self.buffer = Some(buffer);
        self.pixels = Some(pixels);
        Ok(())
    }
}

fn paint_welcome(pixels: &mut [u8]) {
    for y in 0..HEIGHT as usize {
        for x in 0..WIDTH as usize {
            let offset = (y * WIDTH as usize + x) * 4;
            let (r, g, b) = if y < 76 { (18, 39, 75) } else { (14, 22, 37) };
            pixels[offset..offset + 4].copy_from_slice(&[b, g, r, 255]);
        }
    }

    // Geometric Aurea mark and a restrained welcome layout.
    fill_rect(pixels, 54, 128, 92, 92, [220, 170, 55, 255]);
    fill_rect(pixels, 77, 150, 46, 48, [14, 22, 37, 255]);
    fill_rect(pixels, 86, 160, 28, 10, [220, 170, 55, 255]);
    fill_rect(pixels, 54, 256, 612, 2, [55, 83, 125, 255]);
    fill_rect(pixels, 54, 294, 310, 14, [226, 233, 244, 255]);
    fill_rect(pixels, 54, 324, 420, 8, [116, 145, 181, 255]);
}

fn fill_rect(pixels: &mut [u8], x: usize, y: usize, width: usize, height: usize, color: [u8; 4]) {
    for row in y..(y + height).min(HEIGHT as usize) {
        for column in x..(x + width).min(WIDTH as usize) {
            let offset = (row * WIDTH as usize + column) * 4;
            pixels[offset..offset + 4].copy_from_slice(&color);
        }
    }
}

impl Dispatch<WlBuffer, ()> for WelcomeWindow {
    fn event(
        _state: &mut Self,
        _proxy: &WlBuffer,
        _event: wl_buffer::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlShm, ()> for WelcomeWindow {
    fn event(
        _state: &mut Self,
        _proxy: &WlShm,
        _event: wl_shm::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlShmPool, ()> for WelcomeWindow {
    fn event(
        _state: &mut Self,
        _proxy: &WlShmPool,
        _event: wl_shm_pool::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlSurface, ()> for WelcomeWindow {
    fn event(
        _state: &mut Self,
        _proxy: &WlSurface,
        _event: wl_surface::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<XdgWmBase, ()> for WelcomeWindow {
    fn event(
        _state: &mut Self,
        proxy: &XdgWmBase,
        event: xdg_wm_base::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            proxy.pong(serial);
        }
    }
}

impl Dispatch<XdgSurface, ()> for WelcomeWindow {
    fn event(
        state: &mut Self,
        proxy: &XdgSurface,
        event: xdg_surface::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial } = event {
            proxy.ack_configure(serial);
            if let Err(error) = state.draw(qh) {
                eprintln!("Aurea Welcome: cannot draw initial surface: {error}");
                state.closed = true;
            }
        }
    }
}

impl Dispatch<XdgToplevel, ()> for WelcomeWindow {
    fn event(
        state: &mut Self,
        _proxy: &XdgToplevel,
        event: xdg_toplevel::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let xdg_toplevel::Event::Close = event {
            state.closed = true;
        }
    }
}

delegate_noop!(WelcomeWindow: ignore wl_registry::WlRegistry);
delegate_noop!(WelcomeWindow: ignore wl_compositor::WlCompositor);

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
    toplevel.set_title("AureaOS — Welcome");
    toplevel.set_app_id("org.aureaos.Welcome");
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
