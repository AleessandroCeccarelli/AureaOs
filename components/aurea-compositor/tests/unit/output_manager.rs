// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS test module relocated from production source.

use super::*;

fn sample_output(id: OutputId) -> DisplayOutput {
    let mode = OutputMode::new(1920, 1080, 60_000).unwrap();

    DisplayOutput::new(id, format!("HDMI-{id}"), vec![mode], mode, 1).unwrap()
}

fn output_with_modes(id: OutputId, modes: &[OutputMode], current: OutputMode) -> DisplayOutput {
    DisplayOutput::new(id, format!("HDMI-{id}"), modes.to_vec(), current, 1).unwrap()
}

#[test]
fn manager_starts_empty() {
    let manager = OutputManager::new();
    assert!(manager.registry().is_empty());
    assert!(manager.layout().is_empty());
}

#[test]
fn registers_output_and_position() {
    let mut manager = OutputManager::new();

    assert!(manager.register(sample_output(1), (0, 0)).is_ok());
    assert_eq!(manager.registry().len(), 1);
    assert_eq!(manager.layout().get(1), Some(OutputPosition::new(0, 0)));
}

#[test]
fn rejects_duplicate_output() {
    let mut manager = OutputManager::new();

    manager.register(sample_output(1), (0, 0)).unwrap();
    assert_eq!(
        manager.register(sample_output(1), (1920, 0)),
        Err(OutputManagerError::DuplicateOutput(1))
    );
    assert_eq!(manager.registry().len(), 1);
    assert_eq!(manager.layout().len(), 1);
}

#[test]
fn batch_registration_succeeds() {
    let mut manager = OutputManager::new();

    manager
        .register_batch(vec![
            (sample_output(1), (0, 0)),
            (sample_output(2), (1920, 0)),
        ])
        .unwrap();

    assert_eq!(manager.registry().len(), 2);
    assert_eq!(manager.layout().len(), 2);
    assert_eq!(manager.layout().get(2), Some(OutputPosition::new(1920, 0)));
}

#[test]
fn batch_rejects_duplicate_ids_without_mutation() {
    let mut manager = OutputManager::new();

    let result = manager.register_batch(vec![
        (sample_output(1), (0, 0)),
        (sample_output(1), (1920, 0)),
    ]);

    assert_eq!(result, Err(OutputManagerError::DuplicateOutput(1)));
    assert!(manager.registry().is_empty());
    assert!(manager.layout().is_empty());
}

#[test]
fn batch_rejects_existing_id_without_mutation() {
    let mut manager = OutputManager::new();
    manager.register(sample_output(1), (0, 0)).unwrap();

    let result = manager.register_batch(vec![
        (sample_output(2), (1920, 0)),
        (sample_output(1), (3840, 0)),
    ]);

    assert_eq!(result, Err(OutputManagerError::DuplicateOutput(1)));
    assert_eq!(manager.registry().len(), 1);
    assert_eq!(manager.layout().len(), 1);
    assert!(manager.registry().get(2).is_none());
}

#[test]
fn unregister_removes_output_and_position() {
    let mut manager = OutputManager::new();
    manager.register(sample_output(1), (0, 0)).unwrap();

    assert!(manager.unregister(1));
    assert!(manager.registry().is_empty());
    assert!(manager.layout().is_empty());
    assert!(!manager.unregister(1));
}

#[test]
fn rejects_unsupported_mode_and_zero_scale() {
    let mut manager = OutputManager::new();
    manager.register(sample_output(1), (0, 0)).unwrap();

    let unsupported = OutputMode::new(1280, 720, 60_000).unwrap();

    assert_eq!(
        manager.set_mode(1, unsupported),
        Err(OutputManagerError::InvalidMode {
            output: 1,
            mode: unsupported,
        })
    );

    assert_eq!(
        manager.set_scale(1, 0),
        Err(OutputManagerError::InvalidScale {
            output: 1,
            scale: 0,
        })
    );
}

#[test]
fn reconciliation_preserves_unchanged_output_identity_and_configuration() {
    let mut manager = OutputManager::new();
    let id = (7_u128 << 32) | 1;
    manager.register(sample_output(id), (31, 42)).unwrap();
    let original = manager.registry().get(id).unwrap() as *const DisplayOutput;

    let changes = manager
        .reconcile_outputs(&[id], vec![sample_output(id)])
        .unwrap();

    assert_eq!(changes, OutputReconciliation::default());
    assert_eq!(
        manager.registry().get(id).unwrap() as *const DisplayOutput,
        original
    );
    assert_eq!(manager.layout().get(id), Some(OutputPosition::new(31, 42)));
}

#[test]
fn reconciliation_updates_changed_output_and_only_removes_or_adds_owned_ids() {
    let mut manager = OutputManager::new();
    let changed_id = (7_u128 << 32) | 1;
    let disconnected_id = (7_u128 << 32) | 2;
    let unrelated_id = (8_u128 << 32) | 1;
    manager
        .register(sample_output(changed_id), (31, 42))
        .unwrap();
    manager
        .register(sample_output(disconnected_id), (1951, 42))
        .unwrap();
    manager
        .register(sample_output(unrelated_id), (0, 0))
        .unwrap();
    manager.set_scale(changed_id, 2).unwrap();
    manager.set_enabled(changed_id, false).unwrap();

    let old_mode = OutputMode::new(1920, 1080, 60_000).unwrap();
    let new_mode = OutputMode::new(2560, 1440, 60_000).unwrap();
    let changed = DisplayOutput::new(
        changed_id,
        "HDMI-renamed",
        vec![old_mode, new_mode],
        new_mode,
        1,
    )
    .unwrap();
    let added_id = (7_u128 << 32) | 3;

    let changes = manager
        .reconcile_outputs(
            &[changed_id, disconnected_id],
            vec![changed, sample_output(added_id)],
        )
        .unwrap();

    assert_eq!(changes.updated, vec![changed_id]);
    assert_eq!(changes.added, vec![added_id]);
    assert_eq!(changes.removed, vec![disconnected_id]);
    let output = manager.registry().get(changed_id).unwrap();
    assert_eq!(output.name(), "HDMI-renamed");
    assert_eq!(output.current_mode(), old_mode);
    assert_eq!(output.scale(), 2);
    assert!(!output.is_enabled());
    assert_eq!(
        manager.layout().get(changed_id),
        Some(OutputPosition::new(31, 42))
    );
    assert!(manager.registry().get(disconnected_id).is_none());
    assert!(manager.registry().get(added_id).is_some());
    assert!(manager.registry().get(unrelated_id).is_some());
    assert_eq!(
        manager.layout().get(added_id),
        Some(OutputPosition::new(1951, 0))
    );
}

#[test]
fn reconciliation_adds_after_final_set_when_removing_rightmost_output() {
    let mut manager = OutputManager::new();
    let retained = 1;
    let removed = 2;
    manager.register(sample_output(retained), (0, 0)).unwrap();
    manager.register(sample_output(removed), (1920, 0)).unwrap();

    let added = 3;
    let changes = manager
        .reconcile_outputs(
            &[retained, removed],
            vec![sample_output(retained), sample_output(added)],
        )
        .unwrap();

    assert_eq!(changes.removed, vec![removed]);
    assert_eq!(
        manager.layout().get(added),
        Some(OutputPosition::new(1920, 0))
    );
    assert_eq!(
        manager.layout().get(retained),
        Some(OutputPosition::new(0, 0))
    );
}

#[test]
fn reconciliation_uses_replacement_width_when_placing_addition() {
    let mut manager = OutputManager::new();
    let updated_id = 1;
    manager
        .register(sample_output(updated_id), (100, 20))
        .unwrap();
    let replacement_mode = OutputMode::new(2560, 1440, 60_000).unwrap();
    let replacement = DisplayOutput::new(
        updated_id,
        "HDMI-updated",
        vec![replacement_mode],
        replacement_mode,
        1,
    )
    .unwrap();

    manager
        .reconcile_outputs(&[updated_id], vec![replacement, sample_output(2)])
        .unwrap();

    assert_eq!(
        manager.layout().get(updated_id),
        Some(OutputPosition::new(100, 20))
    );
    assert_eq!(manager.layout().get(2), Some(OutputPosition::new(2660, 0)));
}

#[test]
fn reconciliation_preserves_wider_selected_mode_for_layout() {
    let mut manager = OutputManager::new();
    let id = 1;
    let narrow = OutputMode::new(1280, 720, 60_000).unwrap();
    let wide = OutputMode::new(2560, 1440, 60_000).unwrap();
    manager
        .register(output_with_modes(id, &[narrow, wide], wide), (40, 10))
        .unwrap();

    manager
        .reconcile_outputs(
            &[id],
            vec![
                output_with_modes(id, &[narrow, wide], narrow),
                sample_output(2),
            ],
        )
        .unwrap();

    let retained = manager.registry().get(id).unwrap();
    assert_eq!(retained.current_mode(), wide);
    assert_eq!(retained.current_mode().width(), 2560);
    assert_eq!(manager.layout().get(id), Some(OutputPosition::new(40, 10)));
    assert_eq!(manager.layout().get(2), Some(OutputPosition::new(2600, 0)));
}

#[test]
fn reconciliation_preserves_narrower_selected_mode_for_layout() {
    let mut manager = OutputManager::new();
    let id = 1;
    let narrow = OutputMode::new(1280, 720, 60_000).unwrap();
    let wide = OutputMode::new(2560, 1440, 60_000).unwrap();
    manager
        .register(output_with_modes(id, &[narrow, wide], narrow), (25, 5))
        .unwrap();

    manager
        .reconcile_outputs(
            &[id],
            vec![
                output_with_modes(id, &[narrow, wide], wide),
                sample_output(2),
            ],
        )
        .unwrap();

    let retained = manager.registry().get(id).unwrap();
    assert_eq!(retained.current_mode(), narrow);
    assert_eq!(manager.layout().get(id), Some(OutputPosition::new(25, 5)));
    assert_eq!(manager.layout().get(2), Some(OutputPosition::new(1305, 0)));
}

#[test]
fn reconciliation_uses_discovered_current_mode_when_previous_mode_disappears() {
    let mut manager = OutputManager::new();
    let id = 1;
    let previous = OutputMode::new(2560, 1440, 60_000).unwrap();
    let discovered = OutputMode::new(1600, 900, 60_000).unwrap();
    manager
        .register(output_with_modes(id, &[previous], previous), (50, 0))
        .unwrap();

    manager
        .reconcile_outputs(
            &[id],
            vec![
                output_with_modes(id, &[discovered], discovered),
                sample_output(2),
            ],
        )
        .unwrap();

    assert_eq!(
        manager.registry().get(id).unwrap().current_mode(),
        discovered
    );
    assert_eq!(
        manager.registry().get(id).unwrap().current_mode().width(),
        1600
    );
    assert_eq!(manager.layout().get(2), Some(OutputPosition::new(1650, 0)));
}

#[test]
fn reconciliation_preserves_multiple_retained_modes_positions_and_unrelated_output() {
    let mut manager = OutputManager::new();
    let first = 1;
    let second = 2;
    let unrelated = 9;
    let first_mode = OutputMode::new(1600, 900, 60_000).unwrap();
    let second_mode = OutputMode::new(1280, 720, 60_000).unwrap();
    manager
        .register(
            output_with_modes(first, &[first_mode], first_mode),
            (100, 30),
        )
        .unwrap();
    manager
        .register(
            output_with_modes(second, &[second_mode], second_mode),
            (1900, 45),
        )
        .unwrap();
    manager
        .register(sample_output(unrelated), (5000, 100))
        .unwrap();

    manager
        .reconcile_outputs(
            &[first, second],
            vec![
                output_with_modes(first, &[first_mode], first_mode),
                output_with_modes(second, &[second_mode], second_mode),
                sample_output(3),
            ],
        )
        .unwrap();

    assert_eq!(
        manager.registry().get(first).unwrap().current_mode(),
        first_mode
    );
    assert_eq!(
        manager.registry().get(second).unwrap().current_mode(),
        second_mode
    );
    assert_eq!(
        manager.layout().get(first),
        Some(OutputPosition::new(100, 30))
    );
    assert_eq!(
        manager.layout().get(second),
        Some(OutputPosition::new(1900, 45))
    );
    assert_eq!(
        manager.layout().get(unrelated),
        Some(OutputPosition::new(5000, 100))
    );
    assert_eq!(manager.layout().get(3), Some(OutputPosition::new(6920, 0)));
}

#[test]
fn reconciliation_does_not_move_unrelated_outputs_and_avoids_them_for_additions() {
    let mut manager = OutputManager::new();
    let owned_id = 1;
    let unrelated_id = 2;
    manager.register(sample_output(owned_id), (0, 0)).unwrap();
    manager
        .register(sample_output(unrelated_id), (5000, 100))
        .unwrap();

    manager
        .reconcile_outputs(&[owned_id], vec![sample_output(owned_id), sample_output(3)])
        .unwrap();

    assert_eq!(
        manager.layout().get(unrelated_id),
        Some(OutputPosition::new(5000, 100))
    );
    assert_eq!(manager.layout().get(3), Some(OutputPosition::new(6920, 0)));
}

#[test]
fn reconciliation_position_overflow_leaves_manager_unchanged() {
    let mut manager = OutputManager::new();
    let retained = 1;
    manager
        .register(sample_output(retained), (i32::MAX - 100, 0))
        .unwrap();
    let before = manager.clone();

    let result =
        manager.reconcile_outputs(&[retained], vec![sample_output(retained), sample_output(2)]);

    assert_eq!(result, Err(OutputManagerError::PositionOverflow));
    assert_eq!(manager.registry().len(), before.registry().len());
    for output in before.registry().iter() {
        assert_eq!(manager.registry().get(output.id()), Some(output));
        assert_eq!(
            manager.layout().get(output.id()),
            before.layout().get(output.id())
        );
    }
}

#[test]
fn failed_reconciliation_validation_leaves_manager_unchanged() {
    let mut manager = OutputManager::new();
    let existing_id = (7_u128 << 32) | 1;
    manager
        .register(sample_output(existing_id), (31, 42))
        .unwrap();
    let before = manager.clone();

    let invalid_scope = manager.reconcile_outputs(&[], vec![sample_output(existing_id)]);

    assert_eq!(
        invalid_scope,
        Err(OutputManagerError::DuplicateOutput(existing_id))
    );
    assert_eq!(
        manager.registry().get(existing_id),
        before.registry().get(existing_id)
    );
    assert_eq!(
        manager.layout().get(existing_id),
        before.layout().get(existing_id)
    );
}
