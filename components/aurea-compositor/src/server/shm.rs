// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Wayland shared-memory protocol integration.

use smithay::{
    delegate_shm,
    reexports::wayland_server::protocol::wl_buffer::WlBuffer,
    wayland::{
        buffer::BufferHandler,
        shm::{ShmHandler, ShmState},
    },
};

use crate::AureaCompositor;

impl BufferHandler for AureaCompositor {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) {
        // The compositor does not retain imported SHM buffers yet.
        // Vulkan texture-cache invalidation will be added with buffer importing.
    }
}

impl ShmHandler for AureaCompositor {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

delegate_shm!(AureaCompositor);
