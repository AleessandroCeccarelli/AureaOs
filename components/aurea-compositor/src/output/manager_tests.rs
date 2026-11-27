// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use super::*;
use crate::output::OutputMode;
use crate::output::coordinates::{LogicalPoint, LogicalSize};

#[test]
fn logical_bounds_follow_position_and_integer_scale() {
    let mode = OutputMode::new(1920, 1080, 60_000).unwrap();
    let output = DisplayOutput::new(7, "DP-1", vec![mode], mode, 2).unwrap();
    let mut manager = OutputManager::new();
    manager.register(output, (-1920, 40)).unwrap();

    let bounds = manager.logical_bounds(7).unwrap().unwrap();
    assert_eq!(bounds.origin(), LogicalPoint { x: -1920, y: 40 });
    assert_eq!(
        bounds.size(),
        LogicalSize {
            width: 960,
            height: 540
        }
    );
}

#[test]
fn logical_bounds_use_fractional_scale() {
    let mode = OutputMode::new(2560, 1440, 60_000).unwrap();
    let mut output = DisplayOutput::new(8, "DP-2", vec![mode], mode, 1).unwrap();
    output.set_fractional_scale(super::super::scale::ScaleFactor::from_percent(150).unwrap());
    let mut manager = OutputManager::new();
    manager.register(output, (100, -20)).unwrap();

    let bounds = manager.logical_bounds(8).unwrap().unwrap();
    assert_eq!(bounds.origin(), LogicalPoint { x: 100, y: -20 });
    assert_eq!(
        bounds.size(),
        LogicalSize {
            width: 1707,
            height: 960
        }
    );
}

#[test]
fn logical_bounds_swap_dimensions_for_quarter_turn_outputs() {
    let mode = OutputMode::new(1920, 1080, 60_000).unwrap();
    let mut output = DisplayOutput::new(10, "DP-3", vec![mode], mode, 1).unwrap();
    output.set_transform(smithay::utils::Transform::_90);

    let mut manager = OutputManager::new();
    manager.register(output, (40, 70)).unwrap();

    let bounds = manager.logical_bounds(10).unwrap().unwrap();
    assert_eq!(bounds.origin(), LogicalPoint { x: 40, y: 70 });
    assert_eq!(
        bounds.size(),
        LogicalSize {
            width: 1080,
            height: 1920,
        }
    );
}

#[test]
fn disabled_output_has_no_active_bounds() {
    let mode = OutputMode::new(1280, 720, 60_000).unwrap();
    let mut output = DisplayOutput::new(9, "HDMI-A-1", vec![mode], mode, 1).unwrap();
    output.set_enabled(false);
    let mut manager = OutputManager::new();
    manager.register(output, (0, 0)).unwrap();

    assert_eq!(manager.logical_bounds(9).unwrap(), None);
}

#[test]
fn unknown_output_returns_an_error() {
    let manager = OutputManager::new();
    assert_eq!(
        manager.logical_bounds(404).unwrap_err(),
        OutputManagerError::UnknownOutput(404)
    );
}

#[test]
fn configuration_commits_mode_scale_transform_and_position_together() {
    let original = OutputMode::new(1920, 1080, 60_000).unwrap();
    let smaller = OutputMode::new(1280, 720, 60_000).unwrap();
    let output = DisplayOutput::new(11, "DP-4", vec![original, smaller], original, 1).unwrap();
    let mut manager = OutputManager::new();
    manager.register(output, (0, 0)).unwrap();

    manager
        .configure_output(
            11,
            OutputConfiguration {
                mode: Some(smaller),
                scale: Some(OutputScale::Integer(2)),
                transform: Some(smithay::utils::Transform::_90),
                enabled: None,
                position: Some((120, 80)),
            },
        )
        .unwrap();

    let configured = manager.registry().get(11).unwrap();
    assert_eq!(configured.current_mode(), smaller);
    assert_eq!(configured.scale(), 2);
    assert_eq!(configured.transform(), smithay::utils::Transform::_90);
    let bounds = manager.logical_bounds(11).unwrap().unwrap();
    assert_eq!(bounds.origin(), LogicalPoint { x: 120, y: 80 });
    assert_eq!(
        bounds.size(),
        LogicalSize {
            width: 360,
            height: 640
        }
    );
}

#[test]
fn invalid_configuration_rolls_back_every_property() {
    let mode = OutputMode::new(1920, 1080, 60_000).unwrap();
    let unsupported_mode = OutputMode::new(2560, 1440, 60_000).unwrap();
    let output = DisplayOutput::new(12, "DP-5", vec![mode], mode, 1).unwrap();
    let mut manager = OutputManager::new();
    manager.register(output, (10, 20)).unwrap();
    let before = manager.clone();

    let result = manager.configure_output(
        12,
        OutputConfiguration {
            mode: Some(unsupported_mode),
            scale: Some(OutputScale::Integer(2)),
            transform: Some(smithay::utils::Transform::_90),
            position: Some((120, 80)),
            ..OutputConfiguration::default()
        },
    );

    assert_eq!(
        result,
        Err(OutputManagerError::InvalidMode {
            output: 12,
            mode: unsupported_mode,
        })
    );
    assert_eq!(manager.registry().get(12), before.registry().get(12));
    assert_eq!(manager.layout().get(12), before.layout().get(12));
}

#[test]
fn position_overflow_is_rejected_without_mutating_output() {
    let mode = OutputMode::new(1920, 1080, 60_000).unwrap();
    let output = DisplayOutput::new(14, "DP-6", vec![mode], mode, 1).unwrap();
    let mut manager = OutputManager::new();
    manager.register(output, (10, 20)).unwrap();
    let before = manager.clone();

    assert_eq!(
        manager.configure_output(
            14,
            OutputConfiguration {
                position: Some((i32::MAX, 0)),
                ..OutputConfiguration::default()
            },
        ),
        Err(OutputManagerError::PositionOverflow)
    );
    assert_eq!(manager.registry().get(14), before.registry().get(14));
    assert_eq!(manager.layout().get(14), before.layout().get(14));
}

#[test]
fn invalid_integer_scale_does_not_mutate_output() {
    let mode = OutputMode::new(1280, 720, 60_000).unwrap();
    let output = DisplayOutput::new(13, "HDMI-A-2", vec![mode], mode, 1).unwrap();
    let mut manager = OutputManager::new();
    manager.register(output, (0, 0)).unwrap();

    assert_eq!(
        manager.configure_output(
            13,
            OutputConfiguration {
                scale: Some(OutputScale::Integer(0)),
                ..OutputConfiguration::default()
            },
        ),
        Err(OutputManagerError::InvalidScale {
            output: 13,
            scale: 0,
        })
    );
    assert_eq!(manager.registry().get(13).unwrap().scale(), 1);
}

#[test]
fn staged_configuration_does_not_mutate_live_manager_until_installed() {
    let original = OutputMode::new(1920, 1080, 60_000).unwrap();
    let requested = OutputMode::new(1280, 720, 60_000).unwrap();
    let output = DisplayOutput::new(15, "DP-7", vec![original, requested], original, 1).unwrap();
    let mut live = OutputManager::new();
    live.register(output, (0, 0)).unwrap();

    let staged = live
        .staged_configuration(
            15,
            OutputConfiguration {
                mode: Some(requested),
                ..OutputConfiguration::default()
            },
        )
        .unwrap();

    assert_eq!(live.registry().get(15).unwrap().current_mode(), original);
    assert_eq!(staged.registry().get(15).unwrap().current_mode(), requested);

    // The caller installs the candidate only after physical KMS confirmation.
    live = staged;
    assert_eq!(live.registry().get(15).unwrap().current_mode(), requested);
}

#[test]
fn staged_multi_output_configuration_is_atomic_and_keeps_live_state_unchanged() {
    let original = OutputMode::new(1920, 1080, 60_000).unwrap();
    let requested = OutputMode::new(1280, 720, 60_000).unwrap();
    let mut manager = OutputManager::new();
    manager
        .register(
            DisplayOutput::new(21, "DP-1", vec![original, requested], original, 1).unwrap(),
            (0, 0),
        )
        .unwrap();
    manager
        .register(
            DisplayOutput::new(22, "HDMI-A-1", vec![original, requested], original, 1).unwrap(),
            (1920, 0),
        )
        .unwrap();

    let staged = manager
        .staged_configurations(&[
            (
                21,
                OutputConfiguration {
                    mode: Some(requested),
                    position: Some((0, 0)),
                    ..OutputConfiguration::default()
                },
            ),
            (
                22,
                OutputConfiguration {
                    mode: Some(requested),
                    position: Some((1280, 0)),
                    ..OutputConfiguration::default()
                },
            ),
        ])
        .unwrap();

    assert_eq!(manager.registry().get(21).unwrap().current_mode(), original);
    assert_eq!(manager.registry().get(22).unwrap().current_mode(), original);
    assert_eq!(staged.registry().get(21).unwrap().current_mode(), requested);
    assert_eq!(staged.registry().get(22).unwrap().current_mode(), requested);
    assert_eq!(staged.layout().get(22).unwrap().x(), 1280);
}

#[test]
fn staged_multi_output_configuration_rejects_duplicate_ids_without_mutation() {
    let original = OutputMode::new(1920, 1080, 60_000).unwrap();
    let requested = OutputMode::new(1280, 720, 60_000).unwrap();
    let mut manager = OutputManager::new();
    manager
        .register(
            DisplayOutput::new(23, "DP-2", vec![original, requested], original, 1).unwrap(),
            (0, 0),
        )
        .unwrap();

    let result = manager.staged_configurations(&[
        (
            23,
            OutputConfiguration {
                mode: Some(requested),
                ..OutputConfiguration::default()
            },
        ),
        (
            23,
            OutputConfiguration {
                position: Some((100, 100)),
                ..OutputConfiguration::default()
            },
        ),
    ]);

    assert_eq!(result, Err(OutputManagerError::DuplicateOutput(23)));
    assert_eq!(manager.registry().get(23).unwrap().current_mode(), original);
    assert_eq!(manager.layout().get(23).unwrap().x(), 0);
}

