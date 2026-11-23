// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Allocation and attachment of dimensioned Wayland SHM frames.

use std::{
    fs::OpenOptions,
    io,
    os::{fd::AsFd, unix::fs::OpenOptionsExt},
    path::PathBuf,
};

use wayland_client::{QueueHandle, protocol::wl_shm::Format};

use super::{HEIGHT, WIDTH, WelcomeWindow, frame::FrameResources, scaling::render_scaled};

impl FrameResources {
    fn create(
        state: &WelcomeWindow,
        width: usize,
        height: usize,
        qh: &QueueHandle<WelcomeWindow>,
    ) -> io::Result<Self> {
        let stride = width
            .checked_mul(4)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| io::Error::other("Welcome frame stride is too large"))?;
        let buffer_size = (stride as usize)
            .checked_mul(height)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| io::Error::other("Welcome frame allocation is too large"))?;

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
        file.set_len(u64::from(buffer_size))?;

        let mut pixels = unsafe { memmap2::MmapMut::map_mut(&file)? };
        pixels.copy_from_slice(&render_scaled(width, height));

        let pool = state
            .shm
            .create_pool(file.as_fd(), buffer_size as i32, qh, ());
        let buffer = pool.create_buffer(
            0,
            width as i32,
            height as i32,
            stride as i32,
            Format::Argb8888,
            qh,
            (),
        );

        Ok(Self {
            buffer,
            _pool: pool,
            _pixels: pixels,
            size: (width, height),
        })
    }
}

impl WelcomeWindow {
    pub(super) fn draw(&mut self, qh: &QueueHandle<Self>) -> io::Result<()> {
        let (width, height) = self
            .configured_size
            .unwrap_or((WIDTH as i32, HEIGHT as i32));
        let (width, height) = super::dimensions::resolve_size(width, height);

        if self.current_size == Some((width, height)) {
            return Ok(());
        }

        let frame = FrameResources::create(self, width, height, qh)?;
        self.surface.attach(Some(&frame.buffer), 0, 0);
        self.surface.damage(0, 0, width as i32, height as i32);
        self.surface.commit();

        self.current_size = Some(frame.size);
        self.frames.push(frame);
        Ok(())
    }

    pub(super) fn release_frame(
        &mut self,
        released: &wayland_client::protocol::wl_buffer::WlBuffer,
    ) {
        self.frames.retain(|frame| frame.buffer != *released);
    }
}
