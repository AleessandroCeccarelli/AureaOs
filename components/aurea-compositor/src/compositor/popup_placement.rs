// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Output-aware placement for XDG popup surfaces.

use smithay::{
    desktop::{PopupKind, PopupManager, Window, find_popup_root_surface},
    utils::{Logical, Point, Rectangle, Size},
    wayland::shell::xdg::{PopupSurface, PositionerState},
};

use crate::{
    output::coordinates::{LogicalPoint, OutputLogicalBounds},
    AureaCompositor,
};

/// Applies the XDG constraint-adjustment policy against the output containing
/// the popup anchor. Nested popup parents are resolved in toplevel coordinates.
pub(crate) fn constrain_geometry(
    compositor: &AureaCompositor,
    popup: &PopupSurface,
    positioner: PositionerState,
) -> Rectangle<i32, Logical> {
    let fallback = positioner.get_geometry();
    let popup_kind = PopupKind::Xdg(popup.clone());
    let Ok(root) = find_popup_root_surface(&popup_kind) else {
        return fallback;
    };
    let Some((toplevel, toplevel_location)) = compositor
        .window_manager
        .mapped_toplevels_with_location()
        .find(|(toplevel, _)| toplevel.wl_surface() == &root)
    else {
        return fallback;
    };

    let window_geometry = Window::new_wayland_window(toplevel.clone()).geometry();
    let root_window_origin = Point::<i32, Logical>::from(toplevel_location) + window_geometry.loc;
    let parent = match popup.get_parent_surface() {
        Some(parent) => parent,
        None => return fallback,
    };

    let (parent_origin, actual_parent_size) = if parent == root {
        (root_window_origin, window_geometry.size)
    } else {
        let Some((parent_popup, parent_offset)) = PopupManager::popups_for_surface(&root)
            .find(|(candidate, _)| candidate.wl_surface() == &parent)
        else {
            return fallback;
        };
        let parent_geometry = parent_popup.geometry();
        (
            Point::<i32, Logical>::from(toplevel_location) + parent_offset
                - parent_geometry.loc,
            parent_geometry.size,
        )
    };

    let parent_size = positioner.parent_size.unwrap_or(actual_parent_size);
    let anchor = positioner.get_anchor_point();
    let anchor_point = LogicalPoint {
        x: i64::from(parent_origin.x) + i64::from(anchor.x),
        y: i64::from(parent_origin.y) + i64::from(anchor.y),
    };
    let parent_center = LogicalPoint {
        x: i64::from(parent_origin.x) + i64::from(parent_size.w / 2),
        y: i64::from(parent_origin.y) + i64::from(parent_size.h / 2),
    };

    let Some(bounds) = select_constraint_output(compositor, anchor_point, parent_center) else {
        return fallback;
    };
    let Some(target) = target_relative_to_parent(bounds, parent_origin) else {
        return fallback;
    };
    positioner.get_unconstrained_geometry(target)
}

fn select_constraint_output(
    compositor: &AureaCompositor,
    anchor: LogicalPoint,
    parent_center: LogicalPoint,
) -> Option<OutputLogicalBounds> {
    let outputs = compositor
        .output_manager
        .registry()
        .iter()
        .filter(|output| output.is_enabled())
        .filter_map(|output| compositor.output_manager.logical_bounds(output.id()).ok().flatten())
        .collect::<Vec<_>>();

    outputs
        .iter()
        .copied()
        .find(|bounds| bounds.contains(anchor))
        .or_else(|| outputs.iter().copied().find(|bounds| bounds.contains(parent_center)))
}

fn target_relative_to_parent(
    bounds: OutputLogicalBounds,
    parent_origin: Point<i32, Logical>,
) -> Option<Rectangle<i32, Logical>> {
    let origin = bounds.origin();
    let size = bounds.size();
    let x = i32::try_from(origin.x.checked_sub(i64::from(parent_origin.x))?).ok()?;
    let y = i32::try_from(origin.y.checked_sub(i64::from(parent_origin.y))?).ok()?;
    let width = i32::try_from(size.width).ok()?;
    let height = i32::try_from(size.height).ok()?;

    Some(Rectangle::new(
        Point::from((x, y)),
        Size::from((width, height)),
    ))
}

#[cfg(test)]
mod tests {
    use super::target_relative_to_parent;
    use crate::output::coordinates::{LogicalPoint, LogicalSize, OutputLogicalBounds};

    #[test]
    fn converts_global_output_bounds_to_parent_window_coordinates() {
        let bounds = OutputLogicalBounds::from_logical_size(
            LogicalPoint { x: 1920, y: -120 },
            LogicalSize {
                width: 1280,
                height: 720,
            },
        )
        .unwrap();

        let target = target_relative_to_parent(bounds, (2000, 80).into()).unwrap();
        assert_eq!(target.loc, (-80, -200).into());
        assert_eq!(target.size, (1280, 720).into());
    }

    #[test]
    fn rejects_relative_coordinates_that_overflow_protocol_geometry() {
        let bounds = OutputLogicalBounds::from_logical_size(
            LogicalPoint { x: i64::MAX - 100, y: 0 },
            LogicalSize {
                width: 50,
                height: 50,
            },
        )
        .unwrap();

        assert!(target_relative_to_parent(bounds, (i32::MIN, 0).into()).is_none());
    }
}

/// Tracks the last client positioner for each live popup. Smithay exposes the
/// committed popup geometry, while this registry retains the positioner needed
/// to recompute reactive popups after parent/output geometry changes.
#[derive(Default)]
pub(crate) struct PopupPlacementRegistry {
    entries: Vec<(PopupSurface, PositionerState)>,
}

impl PopupPlacementRegistry {
    pub(crate) fn register(&mut self, popup: PopupSurface, positioner: PositionerState) {
        self.update(popup, positioner);
    }

    pub(crate) fn update(&mut self, popup: PopupSurface, positioner: PositionerState) {
        self.entries.retain(|(entry, _)| entry.alive());
        if let Some((_, current)) = self.entries.iter_mut().find(|(entry, _)| entry == &popup) {
            *current = positioner;
        } else {
            self.entries.push((popup, positioner));
        }
    }

    pub(crate) fn remove(&mut self, popup: &PopupSurface) {
        self.entries.retain(|(entry, _)| entry.alive() && entry != popup);
    }

    fn reactive_popups(&mut self) -> Vec<(PopupSurface, PositionerState)> {
        self.entries.retain(|(popup, _)| popup.alive());
        self.entries
            .iter()
            .filter(|(_, positioner)| positioner.reactive)
            .cloned()
            .collect()
    }
}

impl AureaCompositor {
    /// Recomputes all live reactive XDG popups after their parent or output
    /// geometry changes. Configure events are sent only when geometry changed.
    pub(crate) fn reconstrain_reactive_popups(&mut self) {
        let popups = self.popup_placement_registry.reactive_popups();
        let mut changed = false;

        for (popup, positioner) in popups {
            let geometry = constrain_geometry(self, &popup, positioner);
            popup.with_pending_state(|state| {
                state.positioner = positioner;
                state.geometry = geometry;
            });

            match popup.send_pending_configure() {
                Ok(Some(_serial)) => changed = true,
                Ok(None) => {}
                Err(error) => {
                    tracing::debug!(
                        ?error,
                        "Reactive XDG popup could not be reconfigured"
                    );
                }
            }
        }

        if changed {
            self.request_all_output_frames();
        }
    }

    /// Moves a managed toplevel and immediately reconstrains its reactive popups.
    ///
    /// Window move/drag implementations should use this compositor-level entry
    /// point rather than mutating WindowManager directly.
    pub fn set_toplevel_location(
        &mut self,
        surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
        location: (i32, i32),
    ) -> bool {
        if !self.window_manager.set_toplevel_location(surface, location) {
            return false;
        }

        self.reconcile_surface_outputs();
        self.refresh_pointer_focus_at_current_location();
        self.reconstrain_reactive_popups();
        self.request_all_output_frames();
        true
    }
}

