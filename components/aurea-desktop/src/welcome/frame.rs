// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Ownership of client-side Wayland frame resources.

use memmap2::MmapMut;
use wayland_client::protocol::{wl_buffer::WlBuffer, wl_shm_pool::WlShmPool};

/// Keeps every resource backing a submitted frame alive until buffer release.
pub(super) struct FrameResources {
    pub(super) buffer: WlBuffer,
    pub(super) _pool: WlShmPool,
    pub(super) _pixels: MmapMut,
    pub(super) size: (usize, usize),
}
