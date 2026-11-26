// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use crate::AureaCompositor;
use aurea_desktop_api::desktop_interaction::{DesktopInteraction, desktop_interaction_channel};
use smithay::reexports::wayland_server::Display;

#[test]
fn compositor_initializes_wayland_shared_memory_state() {
    let display = Display::<AureaCompositor>::new()
        .expect("Wayland display should initialize for the SHM protocol test");

    let (interactions, _receiver) = desktop_interaction_channel();
    let compositor = AureaCompositor::new(&display.handle(), interactions);

    // Construction registers the mandatory ARGB8888 and XRGB8888 wl_shm
    // formats through Smithay's ShmState global.
    drop(compositor);
}

#[test]
fn render_scene_captures_the_shared_desktop_selection() {
    let display = Display::<AureaCompositor>::new()
        .expect("Wayland display should initialize for the selection snapshot test");
    let (interactions, _receiver) = desktop_interaction_channel();
    let compositor = AureaCompositor::new(&display.handle(), interactions.clone());

    interactions
        .send(DesktopInteraction::DesktopItemSelected {
            id: "system:trash".to_owned(),
        })
        .expect("the shell interaction receiver should remain connected");

    let scene = compositor.render_scene();
    assert_eq!(
        scene.desktop_items().selected_item_id(),
        Some("system:trash")
    );
}
