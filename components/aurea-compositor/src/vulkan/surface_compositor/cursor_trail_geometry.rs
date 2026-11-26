// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Resolves output-local geometry for each historical cursor impression.

use smithay::reexports::ash::vk;

use crate::render::RenderScene;
use crate::renderer::RenderTarget;

use super::cursor_geometry::native_cursor_geometry_at;

/// One clipped, output-local trail impression ready for Vulkan submission.
pub(super) struct CursorTrailDraw {
    pub viewport: vk::Viewport,
    pub scissor: vk::Rect2D,
    pub texture_transform: u32,
    pub opacity: f32,
}

/// Converts scene trail samples into per-output draw geometry.
/// Positions outside this output are omitted by the shared cursor geometry path.
pub(super) fn cursor_trail_draws(
    scene: &RenderScene<'_>,
    target: &RenderTarget,
    image_size: (u32, u32),
    hotspot: (u32, u32),
) -> Vec<CursorTrailDraw> {
    scene.cursor_trail().iter().rev().filter_map(|sample| {
        let (geometry, scissor, texture_transform) = native_cursor_geometry_at(
            sample.position(),
            target,
            image_size,
            hotspot,
        )?;
        Some(CursorTrailDraw {
            viewport: vk::Viewport {
                x: geometry.x() as f32,
                y: geometry.y() as f32,
                width: geometry.width() as f32,
                height: geometry.height() as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            },
            scissor: vk::Rect2D {
                offset: vk::Offset2D {
                    x: i32::try_from(scissor.x).ok()?,
                    y: i32::try_from(scissor.y).ok()?,
                },
                extent: vk::Extent2D {
                    width: scissor.width,
                    height: scissor.height,
                },
            },
            texture_transform,
            opacity: sample.opacity(),
        })
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        input::cursor_trail::CursorTrail,
        output::{DisplayOutput, OutputMode},
        window_manager::WindowManager,
    };
    use smithay::{input::pointer::CursorImageStatus, utils::{Logical, Point}};
    use std::time::{Duration, Instant};

    fn target() -> RenderTarget {
        let mode = OutputMode::new(1920, 1080, 60_000).unwrap();
        let output = DisplayOutput::new(7, "DP-1", vec![mode], mode, 1).unwrap();
        RenderTarget::from_output(&output, (0, 0)).unwrap()
    }

    #[test]
    fn trail_geometry_preserves_hotspot_and_temporal_opacity() {
        let windows = WindowManager::new();
        let start = Instant::now();
        let mut trail = CursorTrail::default();
        trail.record(Point::<f64, Logical>::from((960.0, 540.0)), start);
        let samples = trail.visible_at(start + Duration::from_millis(110));
        let scene = RenderScene::from_window_manager(&windows)
            .with_cursor(&CursorImageStatus::default_named(), Some(Point::from((960.0, 540.0))), 1.0)
            .with_cursor_trail(samples);

        let draws = cursor_trail_draws(&scene, &target(), (32, 32), (5, 3));
        assert_eq!(draws.len(), 1);
        assert!((draws[0].opacity - 0.5).abs() < 0.02);
        assert_eq!(draws[0].scissor.extent.width, 32);
        assert_eq!(draws[0].scissor.extent.height, 32);
    }

    #[test]
    fn trail_impressions_outside_an_output_are_clipped_away() {
        let windows = WindowManager::new();
        let start = Instant::now();
        let mut trail = CursorTrail::default();
        trail.record(Point::<f64, Logical>::from((2200.0, 540.0)), start);
        let scene = RenderScene::from_window_manager(&windows)
            .with_cursor(&CursorImageStatus::default_named(), Some(Point::from((2200.0, 540.0))), 1.0)
            .with_cursor_trail(trail.visible_at(start + Duration::from_millis(10)));

        assert!(cursor_trail_draws(&scene, &target(), (32, 32), (5, 3)).is_empty());
    }
}
