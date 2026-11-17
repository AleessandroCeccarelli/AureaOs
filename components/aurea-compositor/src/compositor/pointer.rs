// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Pointer capability initialization for the compositor seat.

use std::io;

use smithay::input::pointer::PointerHandle;

use super::AureaCompositor;

impl AureaCompositor {
    /// Initializes the seat's Wayland pointer capability.
    ///
    /// Input backends can retrieve the handle after initialization and route
    /// motion, button and axis events through Smithay's pointer state machine.
    pub fn initialize_pointer(&mut self) -> io::Result<()> {
        if self.seat.get_pointer().is_none() {
            let _pointer: PointerHandle<Self> = self.seat.add_pointer();
        }

        Ok(())
    }
}
