// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use super::*;

use crate::drm::output::DrmModeInfo;
use crate::output::OutputMode;

fn descriptor(id: OutputId, width: u32) -> DrmOutputDescriptor {
    let mode = DrmModeInfo::new(width, 1080, 60_000);

    DrmOutputDescriptor::new(id, format!("Display-{id}"), vec![mode], mode, 1)
}

#[test]
fn initial_registration_starts_at_origin() {
    let mut manager = OutputManager::new();

    register_drm_output_descriptors(vec![descriptor(1, 1920), descriptor(2, 1280)], &mut manager)
        .unwrap();

    assert_eq!(manager.layout().get(1).unwrap().x(), 0);
    assert_eq!(manager.layout().get(2).unwrap().x(), 1920);
}

#[test]
fn separate_device_registration_uses_global_layout() {
    let mut manager = OutputManager::new();

    register_drm_output_descriptors(vec![descriptor(1, 1920)], &mut manager).unwrap();

    register_drm_output_descriptors(vec![descriptor(2, 1280)], &mut manager).unwrap();

    assert_eq!(manager.layout().get(1).unwrap().x(), 0);
    assert_eq!(manager.layout().get(2).unwrap().x(), 1920);
}

#[test]
fn new_device_is_placed_after_existing_offset_outputs() {
    let mut manager = OutputManager::new();
    let mode = OutputMode::new(1920, 1080, 60_000).unwrap();

    let existing = crate::output::DisplayOutput::new(10, "Existing", vec![mode], mode, 1).unwrap();

    manager.register(existing, (300, 40)).unwrap();

    register_drm_output_descriptors(vec![descriptor(20, 1280)], &mut manager).unwrap();

    assert_eq!(manager.layout().get(20).unwrap().x(), 2220);
}

#[test]
fn empty_registration_does_not_change_layout() {
    let mut manager = OutputManager::new();
    let mode = OutputMode::new(1920, 1080, 60_000).unwrap();

    manager
        .register(
            crate::output::DisplayOutput::new(1, "Existing", vec![mode], mode, 1).unwrap(),
            (100, 0),
        )
        .unwrap();

    let before = manager.layout().get(1);

    assert_eq!(
        register_drm_output_descriptors(Vec::new(), &mut manager).unwrap(),
        0
    );

    assert_eq!(manager.layout().get(1), before);
}
