// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use super::*;
use crate::output::coordinates::{LogicalPoint, LogicalSize};
use crate::output::OutputMode;

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
    let output = DisplayOutput::new(12, "DP-5", vec![mode], mode, 1).unwrap();
    let mut manager = OutputManager::new();
    manager.register(output, (10, 20)).unwrap();
    let before = manager.clone();

    let result = manager.configure_output(
        12,
        OutputConfiguration {
            scale: Some(OutputScale::Integer(2)),
            transform: Some(smithay::utils::Transform::_90),
            position: Some((i32::MAX, 0)),
            ..OutputConfiguration::default()
        },
    );

    assert_eq!(result, Err(OutputManagerError::PositionOverflow));
    assert_eq!(manager.registry().get(12), before.registry().get(12));
    assert_eq!(manager.layout().get(12), before.layout().get(12));
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
