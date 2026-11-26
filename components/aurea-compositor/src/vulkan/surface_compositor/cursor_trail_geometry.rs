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
    scene.cursor_trail().iter().filter_map(|sample| {
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
