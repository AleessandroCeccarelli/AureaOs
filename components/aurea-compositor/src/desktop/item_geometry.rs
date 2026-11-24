// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Per-output geometry for native desktop item artwork.

use aurea_desktop_api::desktop::DesktopItemId;
use smithay::utils::Transform;

use crate::{
    desktop::DesktopItemsSnapshot,
    render::geometry::{
        SurfaceGeometry, SurfaceScissor, surface_geometry,
        surface_geometry_with_fractional_scale, transform_geometry_for_output,
    },
    renderer::RenderTarget,
};

/// Native desktop artwork is laid out in logical pixels, independent of output DPI.
pub(crate) const DESKTOP_ICON_LOGICAL_SIZE: u32 = 64;

/// One desktop icon clipped to a single physical output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DesktopItemDrawGeometry {
    pub(crate) item_id: DesktopItemId,
    pub(crate) is_selected: bool,
    pub(crate) geometry: SurfaceGeometry,
    pub(crate) scissor: SurfaceScissor,
}

/// Resolves desktop item artwork against one output.
///
/// Positions are global logical coordinates. Each output gets its own physical
/// destination, exact fractional scale when configured, output transform and
/// clipping rectangle. Items outside this output are omitted.
pub(crate) fn desktop_item_geometries(
    snapshot: &DesktopItemsSnapshot,
    target: &RenderTarget,
) -> Vec<DesktopItemDrawGeometry> {
    let mut result = Vec::new();

    for item in snapshot.items() {
        let position = item.position();
        let Some(geometry) = (if let Some(scale) = target.fractional_scale() {
            surface_geometry_with_fractional_scale(
                (position.x, position.y),
                (DESKTOP_ICON_LOGICAL_SIZE, DESKTOP_ICON_LOGICAL_SIZE),
                1,
                Transform::Normal,
                target.logical_origin(),
                scale,
            )
        } else {
            surface_geometry(
                (position.x, position.y),
                (DESKTOP_ICON_LOGICAL_SIZE, DESKTOP_ICON_LOGICAL_SIZE),
                1,
                Transform::Normal,
                target.logical_origin(),
                target.scale(),
            )
        }) else {
            // A malformed or overflowing saved position must not hide other icons.
            continue;
        };

        let Some(geometry) = transform_geometry_for_output(
            geometry,
            target.width(),
            target.height(),
            target.transform(),
        ) else {
            continue;
        };
        let Some(scissor) = geometry.scissor(target.width(), target.height()) else {
            continue;
        };

        result.push(DesktopItemDrawGeometry {
            item_id: item.id().clone(),
            is_selected: snapshot.is_selected(item.id()),
            geometry,
            scissor,
        });
    }

    result
}

#[cfg(test)]
mod tests {
    use super::{DESKTOP_ICON_LOGICAL_SIZE, desktop_item_geometries};
    use crate::{
        desktop::DesktopItemsSnapshot,
        output::{DisplayOutput, OutputMode},
        renderer::RenderTarget,
    };

    fn target(origin: (i32, i32), scale: u32) -> RenderTarget {
        let mode = OutputMode::new(1920, 1080, 60_000).unwrap();
        let output = DisplayOutput::new(1, "DP-1", vec![mode], mode, scale).unwrap();
        RenderTarget::from_output(&output, origin).unwrap()
    }

    #[test]
    fn selected_item_geometry_carries_selection_state() {
        let manager = aurea_desktop_api::desktop::DesktopManager::default();
        let snapshot = DesktopItemsSnapshot::capture(&manager)
            .with_selected_item(Some("system:trash".to_owned()));

        let geometries = desktop_item_geometries(&snapshot, &target((0, 0), 1));
        assert_eq!(geometries.len(), 1);
        assert!(geometries[0].is_selected);
    }

    #[test]
    fn unselected_item_geometry_does_not_mark_selection() {
        let snapshot = DesktopItemsSnapshot::capture(
            &aurea_desktop_api::desktop::DesktopManager::default(),
        );

        let geometries = desktop_item_geometries(&snapshot, &target((0, 0), 1));
        assert_eq!(geometries.len(), 1);
        assert!(!geometries[0].is_selected);
    }

    #[test]
    fn positions_are_translated_into_each_outputs_local_space() {
        let mut manager = aurea_desktop_api::desktop::DesktopManager::default();
        let trash = manager.items()[0].id().clone();
        manager.set_item_position(
            &trash,
            aurea_desktop_api::desktop::DesktopPosition::new(1960, 40),
        ).unwrap();
        let snapshot = DesktopItemsSnapshot::capture(&manager);

        let geometries = desktop_item_geometries(&snapshot, &target((1920, 0), 1));
        assert_eq!(geometries.len(), 1);
        assert_eq!(geometries[0].geometry.x(), 40);
        assert_eq!(geometries[0].geometry.y(), 40);
        assert_eq!(geometries[0].geometry.width(), DESKTOP_ICON_LOGICAL_SIZE);
    }

    #[test]
    fn item_outside_an_output_is_not_emitted_for_that_output() {
        let snapshot = DesktopItemsSnapshot::capture(
            &aurea_desktop_api::desktop::DesktopManager::default(),
        );
        assert!(desktop_item_geometries(&snapshot, &target((1920, 0), 1))
            .is_empty());
    }

    #[test]
    fn icon_geometry_uses_integer_output_scale() {
        let snapshot = DesktopItemsSnapshot::capture(
            &aurea_desktop_api::desktop::DesktopManager::default(),
        );
        let geometries = desktop_item_geometries(&snapshot, &target((0, 0), 2));
        assert_eq!(geometries.len(), 1);
        assert_eq!(geometries[0].geometry.width(), DESKTOP_ICON_LOGICAL_SIZE * 2);
        assert_eq!(geometries[0].geometry.height(), DESKTOP_ICON_LOGICAL_SIZE * 2);
    }

    #[test]
    fn partially_visible_icon_is_clipped_to_the_output() {
        let mut manager = aurea_desktop_api::desktop::DesktopManager::default();
        let trash = manager.items()[0].id().clone();
        manager.set_item_position(
            &trash,
            aurea_desktop_api::desktop::DesktopPosition::new(-32, 12),
        ).unwrap();
        let snapshot = DesktopItemsSnapshot::capture(&manager);

        let geometries = desktop_item_geometries(&snapshot, &target((0, 0), 1));
        assert_eq!(geometries.len(), 1);
        assert_eq!(geometries[0].scissor.x, 0);
        assert_eq!(geometries[0].scissor.width, 32);
    }
}
