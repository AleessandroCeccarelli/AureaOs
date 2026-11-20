// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use super::*;

    use super::*;

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
