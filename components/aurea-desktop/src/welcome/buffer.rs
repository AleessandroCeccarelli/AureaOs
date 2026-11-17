// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Shared-memory buffer allocation and surface attachment.

use std::{
    fs::OpenOptions,
    io,
    os::{fd::AsFd, unix::fs::OpenOptionsExt},
    path::PathBuf,
};

use memmap2::MmapMut;
use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_client::protocol::wl_shm_pool::WlShmPool;
use wayland_client::{Dispatch, QueueHandle};

use super::{HEIGHT, WIDTH, WelcomeWindow, painter::paint_welcome};

const STRIDE: u32 = WIDTH as u32 * 4;
const BUFFER_SIZE: u32 = STRIDE * HEIGHT as u32;

impl WelcomeWindow {
    pub(super) fn draw(&mut self, qh: &QueueHandle<Self>) -> io::Result<()> {
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
            .mode(0o600)
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
            wayland_client::protocol::wl_shm::Format::Argb8888,
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
