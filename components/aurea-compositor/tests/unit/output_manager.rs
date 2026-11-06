// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS test module relocated from production source.

use super::*;

fn sample_output(id: OutputId) -> DisplayOutput {
    let mode = OutputMode::new(1920, 1080, 60_000).unwrap();

    DisplayOutput::new(id, format!("HDMI-{id}"), vec![mode], mode, 1).unwrap()
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
