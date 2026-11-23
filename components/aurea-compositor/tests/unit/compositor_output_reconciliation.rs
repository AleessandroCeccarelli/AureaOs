// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use aurea_compositor::{
    AureaCompositor,
    output::{DisplayOutput, OutputMode, manager::OutputReconciliation},
};

use aurea_desktop_api::desktop_interaction::desktop_interaction_channel;
use smithay::reexports::wayland_server::Display;

#[test]
fn removed_output_unpublishes_its_wayland_global() {
    let display = Display::<AureaCompositor>::new().unwrap();
    let display_handle = display.handle();
    let (interactions, _receiver) = desktop_interaction_channel();
    let mut compositor = AureaCompositor::new(&display_handle, interactions);

    let output_id = (4_u128 << 32) | 1;
    let initial_mode = OutputMode::new(1920, 1080, 60_000).unwrap();
    let output =
        DisplayOutput::new(output_id, "HDMI-A-1", vec![initial_mode], initial_mode, 1).unwrap();

    compositor
        .output_manager()
        .register(output, (10, 20))
        .unwrap();
    compositor
        .publish_wayland_output(&display_handle, output_id)
        .unwrap();

    assert!(compositor.has_published_wayland_output(output_id));

    let previous = compositor.output_manager().clone();
    assert!(compositor.output_manager().unregister(output_id));

    let changes = OutputReconciliation {
        removed: vec![output_id],
        ..OutputReconciliation::default()
    };

    compositor
        .publish_output_reconciliation(&display_handle, previous, &changes)
        .unwrap();

    assert!(
        compositor
            .output_manager()
            .registry()
            .get(output_id)
            .is_none()
    );
    assert!(!compositor.has_published_wayland_output(output_id));
}
