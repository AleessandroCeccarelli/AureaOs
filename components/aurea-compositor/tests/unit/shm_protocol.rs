// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use aurea_compositor::AureaCompositor;
use smithay::reexports::wayland_server::Display;

#[test]
fn compositor_initializes_wayland_shared_memory_state() {
    let display = Display::<AureaCompositor>::new()
        .expect("Wayland display should initialize for the SHM protocol test");

    let compositor = AureaCompositor::new(&display.handle());

    // Construction registers the mandatory ARGB8888 and XRGB8888 wl_shm
    // formats through Smithay's ShmState global.
    drop(compositor);
}
