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
    assert_eq!(manager.active_output_count(), 0);
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
    assert_eq!(manager.register(sample_output(1), (1920, 0)), Err(OutputManagerError::DuplicateOutput(1)));
    assert_eq!(manager.registry().len(), 1);
    assert_eq!(manager.layout().len(), 1);
}

#[test]
fn batch_registration_succeeds() {
    let mut manager = OutputManager::new();
    manager.register_batch(vec![(sample_output(1), (0, 0)), (sample_output(2), (1920, 0))]).unwrap();
    assert_eq!(manager.registry().len(), 2);
    assert_eq!(manager.layout().len(), 2);
}

#[test]
fn batch_rejects_duplicate_ids_without_mutation() {
    let mut manager = OutputManager::new();
    let result = manager.register_batch(vec![(sample_output(1), (0, 0)), (sample_output(1), (1920, 0))]);
    assert_eq!(result, Err(OutputManagerError::DuplicateOutput(1)));
    assert!(manager.registry().is_empty());
    assert!(manager.layout().is_empty());
}

#[test]
fn batch_rejects_existing_id_without_mutation() {
    let mut manager = OutputManager::new();
    manager.register(sample_output(1), (0, 0)).unwrap();
    let result = manager.register_batch(vec![(sample_output(2), (1920, 0)), (sample_output(1), (3840, 0))]);
    assert_eq!(result, Err(OutputManagerError::DuplicateOutput(1)));
    assert_eq!(manager.registry().len(), 1);
    assert!(manager.registry().get(2).is_none());
}

#[test]
fn rejects_fifth_active_output_without_mutation() {
    let mut manager = OutputManager::new();
    for id in 1..=MAX_ACTIVE_OUTPUTS {
        manager.register(sample_output(id as u128), (id as i32 * 1920, 0)).unwrap();
    }
    let result = manager.register(sample_output(5), (9600, 0));
    assert_eq!(result, Err(OutputManagerError::TooManyActiveOutputs { maximum: MAX_ACTIVE_OUTPUTS }));
    assert_eq!(manager.active_output_count(), MAX_ACTIVE_OUTPUTS);
    assert_eq!(manager.registry().len(), MAX_ACTIVE_OUTPUTS);
}

#[test]
fn batch_rejects_more_than_four_active_outputs_atomically() {
    let mut manager = OutputManager::new();
    let outputs = (1..=MAX_ACTIVE_OUTPUTS + 1)
        .map(|id| (sample_output(id as u128), (id as i32 * 1920, 0)))
        .collect();
    assert_eq!(manager.register_batch(outputs), Err(OutputManagerError::TooManyActiveOutputs { maximum: MAX_ACTIVE_OUTPUTS }));
    assert!(manager.registry().is_empty());
}

#[test]
fn disabled_output_does_not_consume_active_capacity() {
    let mut manager = OutputManager::new();
    for id in 1..=MAX_ACTIVE_OUTPUTS {
        let mut output = sample_output(id as u128);
        output.set_enabled(false);
        manager.register(output, (id as i32 * 1920, 0)).unwrap();
    }
    assert_eq!(manager.active_output_count(), 0);
    manager.register(sample_output(5), (9600, 0)).unwrap();
    assert_eq!(manager.active_output_count(), 1);
}

#[test]
fn enabling_output_respects_active_limit() {
    let mut manager = OutputManager::new();
    for id in 1..=MAX_ACTIVE_OUTPUTS {
        manager.register(sample_output(id as u128), (id as i32 * 1920, 0)).unwrap();
    }
    let mut spare = sample_output(5);
    spare.set_enabled(false);
    manager.register(spare, (9600, 0)).unwrap();
    assert_eq!(manager.set_enabled(5, true), Err(OutputManagerError::TooManyActiveOutputs { maximum: MAX_ACTIVE_OUTPUTS }));
    assert!(!manager.registry().get(5).unwrap().is_enabled());
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
    assert_eq!(manager.set_mode(1, unsupported), Err(OutputManagerError::InvalidMode { output: 1, mode: unsupported }));
    assert_eq!(manager.set_scale(1, 0), Err(OutputManagerError::InvalidScale { output: 1, scale: 0 }));
}

#[test]
fn reconciliation_preserves_unchanged_output_identity_and_configuration() {
    let mut manager = OutputManager::new();
    let id = (7_u128 << 32) | 1;
    manager.register(sample_output(id), (31, 42)).unwrap();
    let original = manager.registry().get(id).unwrap() as *const DisplayOutput;
    let changes = manager.reconcile_outputs(&[id], vec![sample_output(id)]).unwrap();
    assert_eq!(changes, OutputReconciliation::default());
    assert_eq!(manager.registry().get(id).unwrap() as *const DisplayOutput, original);
    assert_eq!(manager.layout().get(id), Some(OutputPosition::new(31, 42)));
}

#[test]
fn reconciliation_updates_changed_output_and_only_removes_or_adds_owned_ids() {
    let mut manager = OutputManager::new();
    let changed_id = (7_u128 << 32) | 1;
    let disconnected_id = (7_u128 << 32) | 2;
    let unrelated_id = (8_u128 << 32) | 1;
    manager.register(sample_output(changed_id), (31, 42)).unwrap();
    manager.register(sample_output(disconnected_id), (1951, 42)).unwrap();
    manager.register(sample_output(unrelated_id), (0, 0)).unwrap();
    manager.set_scale(changed_id, 2).unwrap();
    manager.set_enabled(changed_id, false).unwrap();
    let old_mode = OutputMode::new(1920, 1080, 60_000).unwrap();
    let new_mode = OutputMode::new(2560, 1440, 60_000).unwrap();
    let changed = DisplayOutput::new(changed_id, "HDMI-renamed", vec![old_mode, new_mode], new_mode, 1).unwrap();
    let added_id = (7_u128 << 32) | 3;
    let changes = manager.reconcile_outputs(&[changed_id, disconnected_id], vec![changed, sample_output(added_id)]).unwrap();
    assert_eq!(changes.updated, vec![changed_id]);
    assert_eq!(changes.added, vec![added_id]);
    assert_eq!(changes.removed, vec![disconnected_id]);
    let output = manager.registry().get(changed_id).unwrap();
    assert_eq!(output.name(), "HDMI-renamed");
    assert_eq!(output.current_mode(), old_mode);
    assert_eq!(output.scale(), 2);
    assert!(!output.is_enabled());
    assert_eq!(manager.layout().get(changed_id), Some(OutputPosition::new(31, 42)));
    assert!(manager.registry().get(disconnected_id).is_none());
    assert!(manager.registry().get(added_id).is_some());
    assert_eq!(manager.layout().get(added_id), Some(OutputPosition::new(1951, 0)));
}

#[test]
fn reconciliation_adds_after_final_set_when_removing_rightmost_output() {
    let mut manager = OutputManager::new();
    manager.register(sample_output(1), (0, 0)).unwrap();
    manager.register(sample_output(2), (1920, 0)).unwrap();
    let changes = manager.reconcile_outputs(&[1, 2], vec![sample_output(1), sample_output(3)]).unwrap();
    assert_eq!(changes.removed, vec![2]);
    assert_eq!(manager.layout().get(3), Some(OutputPosition::new(1920, 0)));
}

#[test]
fn reconciliation_uses_replacement_width_when_placing_addition() {
    let mut manager = OutputManager::new();
    let narrow = OutputMode::new(1920, 1080, 60_000).unwrap();
    let wide = OutputMode::new(2560, 1440, 60_000).unwrap();
    manager.register(output_with_modes(1, &[narrow], narrow), (100, 20)).unwrap();
    let replacement = DisplayOutput::new(1, "HDMI-updated", vec![wide], wide, 1).unwrap();
    manager.reconcile_outputs(&[1], vec![replacement, sample_output(2)]).unwrap();
    assert_eq!(manager.layout().get(1), Some(OutputPosition::new(100, 20)));
    assert_eq!(manager.layout().get(2), Some(OutputPosition::new(2660, 0)));
}

#[test]
fn reconciliation_preserves_retained_current_mode_for_layout() {
    let mut manager = OutputManager::new();
    let narrow = OutputMode::new(1280, 720, 60_000).unwrap();
    let wide = OutputMode::new(2560, 1440, 60_000).unwrap();
    manager.register(output_with_modes(1, &[narrow, wide], wide), (40, 10)).unwrap();
    manager.reconcile_outputs(&[1], vec![output_with_modes(1, &[narrow, wide], narrow), sample_output(2)]).unwrap();
    assert_eq!(manager.registry().get(1).unwrap().current_mode(), wide);
    assert_eq!(manager.layout().get(2), Some(OutputPosition::new(2600, 0)));
}

#[test]
fn reconciliation_rejects_fifth_active_output_without_mutation() {
    let mut manager = OutputManager::new();
    let ids = [1, 2, 3, 4];
    for id in ids { manager.register(sample_output(id), (id as i32 * 1920, 0)).unwrap(); }
    let result = manager.reconcile_outputs(&[], vec![sample_output(5)]);
    assert_eq!(result, Err(OutputManagerError::TooManyActiveOutputs { maximum: MAX_ACTIVE_OUTPUTS }));
    assert_eq!(manager.active_output_count(), MAX_ACTIVE_OUTPUTS);
    assert!(manager.registry().get(5).is_none());
}
