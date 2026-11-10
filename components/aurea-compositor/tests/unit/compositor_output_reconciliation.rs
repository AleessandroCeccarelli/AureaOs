
 // SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use aurea_compositor::{
    AureaCompositor,
    output::{
        DisplayOutput,
        OutputMode,
        manager::OutputReconciliation,
    },
};

use smithay::reexports::wayland_server::Display;

#[test]
fn failed_wayland_refresh_keeps_the_previously_published_global() {
    let display = Display::<AureaCompositor>::new().unwrap();
    let display_handle = display.handle();
    let mut compositor = AureaCompositor::new(&display_handle);

    let output_id = (4_u128 << 32) | 1;
    let initial_mode = OutputMode::new(1920, 1080, 60_000).unwrap();

    let initial = DisplayOutput::new(
        output_id,
        "HDMI-A-1",
        vec![initial_mode],
        initial_mode,
        1,
    )
    .unwrap();

    compositor
        .output_manager()
        .register(initial, (10, 20))
        .unwrap();

    compositor
        .publish_wayland_output(&display_handle, output_id)
        .unwrap();

    assert!(compositor.has_published_wayland_output(output_id));

    let previous = compositor.output_manager().clone();

    compositor
        .output_manager()
        .set_scale(output_id, u32::MAX)
        .unwrap();

    let changes = OutputReconciliation {
        updated: vec![output_id],
        ..OutputReconciliation::default()
    };

    assert!(
        compositor
            .publish_output_reconciliation(&display_handle, previous, &changes)
            .is_err()
    );

    assert_eq!(
        compositor
            .output_manager()
            .registry()
            .get(output_id)
            .unwrap()
            .scale(),
        1
    );

    assert!(compositor.has_published_wayland_output(output_id));
}
