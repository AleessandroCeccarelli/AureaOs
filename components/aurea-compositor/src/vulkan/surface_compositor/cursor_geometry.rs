// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Computes the native cursor's physical geometry for one output.

use smithay::utils::Transform;

use crate::{
    render::{
        RenderScene,
        geometry::{
            SurfaceGeometry, SurfaceScissor, compose_surface_transforms, surface_geometry,
            surface_geometry_with_fractional_scale, surface_transform_index,
            transform_geometry_for_output,
        },
    },
    renderer::RenderTarget,
};

/// Resolves the cursor rectangle, clipping and texture transform for an output.
pub(super) fn native_cursor_geometry(
    scene: &RenderScene<'_>,
    target: &RenderTarget,
    image_size: (u32, u32),
    hotspot: (u32, u32),
) -> Option<(SurfaceGeometry, SurfaceScissor, u32)> {
    let pointer = scene.native_cursor_pointer_location()?;
    let output_scale = target.effective_scale();
    if !output_scale.is_finite() || output_scale <= 0.0 {
        return None;
    }
    let logical_width = (f64::from(image_size.0) / output_scale).round().max(1.0) as u32;
    let logical_height = (f64::from(image_size.1) / output_scale).round().max(1.0) as u32;
    let logical_hotspot_x = f64::from(hotspot.0) / output_scale;
    let logical_hotspot_y = f64::from(hotspot.1) / output_scale;
    let location = (
        (pointer.x - logical_hotspot_x).round() as i32,
        (pointer.y - logical_hotspot_y).round() as i32,
    );
    let scaled_location = location;
    let geometry = if let Some(scale) = target.fractional_scale() {
        surface_geometry_with_fractional_scale(
            scaled_location,
            (logical_width, logical_height),
            1,
            Transform::Normal,
            target.logical_origin(),
            scale,
        )
    } else {
        surface_geometry(
            scaled_location,
            (logical_width, logical_height),
            1,
            Transform::Normal,
            target.logical_origin(),
            target.scale(),
        )
    }?;
    let geometry = transform_geometry_for_output(
        geometry,
        target.width(),
        target.height(),
        target.transform(),
    )?;
    let scissor = geometry.scissor(target.width(), target.height())?;
    let texture_transform = surface_transform_index(compose_surface_transforms(
        Transform::Normal,
        target.transform(),
    )?)?;

    Some((geometry, scissor, texture_transform))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        output::{DisplayOutput, OutputMode},
        window_manager::WindowManager,
    };
    use smithay::input::pointer::CursorImageStatus;
    use smithay::utils::{Logical, Point};

    fn target() -> RenderTarget {
        let mode = OutputMode::new(1920, 1080, 60_000).unwrap();
        let output = DisplayOutput::new(7, "DP-1", vec![mode], mode, 1).unwrap();
        RenderTarget::from_output(&output, (0, 0)).unwrap()
    }

    #[test]
    fn default_arrow_has_drawable_geometry_on_the_primary_output() {
        let windows = WindowManager::new();
        let pointer = Point::<f64, Logical>::from((960.0, 540.0));
        let status = CursorImageStatus::default_named();
        let scene = RenderScene::from_window_manager(&windows)
            .with_cursor(&status, Some(pointer), 1.0);

        let geometry = native_cursor_geometry(&scene, &target(), (32, 32), (5, 3));

        assert!(geometry.is_some(), "default arrow must intersect its output");
        let (surface, scissor, _) = geometry.unwrap();
        assert_eq!(surface.width(), 32);
        assert_eq!(surface.height(), 32);
        assert!(scissor.width > 0 && scissor.height > 0);
    }

    #[test]
    fn named_cursor_uses_its_own_hotspot_instead_of_the_arrow_hotspot() {
        let windows = WindowManager::new();
        let pointer = Point::<f64, Logical>::from((960.0, 540.0));
        let status = CursorImageStatus::default_named();
        let scene = RenderScene::from_window_manager(&windows)
            .with_cursor(&status, Some(pointer), 1.0);

        let (geometry, _, _) = native_cursor_geometry(&scene, &target(), (48, 48), (24, 24))
            .expect("cursor intersects the output");

        assert_eq!((geometry.x(), geometry.y()), (936, 516));
        assert_eq!((geometry.width(), geometry.height()), (48, 48));
    }

    #[test]
    fn cursor_on_another_monitor_is_not_drawn_on_this_output() {
        let windows = WindowManager::new();
        let pointer = Point::<f64, Logical>::from((2200.0, 540.0));
        let status = CursorImageStatus::default_named();
        let scene = RenderScene::from_window_manager(&windows)
            .with_cursor(&status, Some(pointer), 1.0);

        assert!(native_cursor_geometry(&scene, &target(), (32, 32), (5, 3)).is_none());
    }
}
