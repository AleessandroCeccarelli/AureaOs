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
        // Surface-owned snapshots contain copied pixels, not a reference to
        // the destroyed client buffer. Surface destruction drops that state.
    }
}

impl ShmHandler for AureaCompositor {
    fn shm_state(&self) -> &ShmState {
        self.wayland_shm_state()
    }
}

delegate_shm!(AureaCompositor);
