// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Collects mapped surfaces into a renderable scene.

pub mod cursor;
pub mod geometry;
pub mod hit_test;
pub mod shm_buffer;
pub mod shm_surface;
pub mod surface_tree;

use surface_tree::{SurfaceTreeSnapshot, collect_surface_tree_ids, collect_surface_tree_snapshots};

use crate::desktop::{DesktopItemsSnapshot, DesktopSnapshot};

use smithay::{
    backend::renderer::{
        ImportAll, Renderer,
        element::{
            Kind,
            surface::{WaylandSurfaceRenderElement, render_elements_from_surface_tree},
        },
    },
    desktop::{WindowSurfaceType, utils::under_from_surface_tree},
    input::pointer::{CursorImageStatus, CursorImageSurfaceData},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{IsAlive, Logical, Physical, Point},
    wayland::{compositor::with_states, shell::xdg::ToplevelSurface},
};

use crate::window_manager::WindowManager;

/// A snapshot of the mapped toplevels and pointer image for one rendering pass.
pub struct RenderScene<'a> {
    desktop: DesktopSnapshot,
    desktop_items: DesktopItemsSnapshot,
    toplevels: Vec<RenderToplevel<'a>>,
    cursor_surface: Option<(WlSurface, Point<i32, Logical>, Point<i32, Logical>)>,
    native_cursor_location: Option<Point<i32, Logical>>,
    native_cursor_scale: f64,
}

struct RenderToplevel<'a> {
    surface: &'a ToplevelSurface,
    location: Point<i32, Logical>,
}

impl<'a> RenderScene<'a> {
    /// Builds a scene from the window manager's currently mapped toplevels.
    pub fn from_window_manager(window_manager: &'a WindowManager) -> Self {
        Self {
            desktop: DesktopSnapshot::new(
                crate::desktop::DesktopSurface::new(
                    crate::desktop::DesktopTheme::Light.desktop_color(),
                ),
                None,
            ),
            desktop_items: DesktopItemsSnapshot::default(),
            toplevels: window_manager
                .mapped_toplevels_with_location()
                .map(|(surface, location)| RenderToplevel { surface, location })
                .collect(),
            cursor_surface: None,
            native_cursor_location: None,
            native_cursor_scale: 1.0,
        }
    }

    /// Replaces the desktop state captured for this render frame.
    pub fn with_desktop(mut self, desktop: DesktopSnapshot) -> Self {
        self.desktop = desktop;
        self
    }

    /// Returns the immutable desktop state captured for this frame.
    pub fn desktop(&self) -> DesktopSnapshot {
        self.desktop
    }

    /// Replaces the desktop-entry collection captured for this render frame.
    pub fn with_desktop_items(mut self, items: DesktopItemsSnapshot) -> Self {
        self.desktop_items = items;
        self
    }

    /// Returns the owned desktop-entry snapshot captured for this frame.
    pub fn desktop_items(&self) -> &DesktopItemsSnapshot {
        &self.desktop_items
    }

    /// Adds the client-provided cursor surface at the pointer's global location.
    ///
    /// Wayland defines the cursor surface origin as pointer location minus its
    /// surface-local hotspot. Keeping it in the scene makes it use the same
    /// validated SHM upload/cache and output clipping path as other surfaces.
    pub fn with_cursor(
        mut self,
        status: &CursorImageStatus,
        pointer_location: Option<Point<f64, Logical>>,
        native_cursor_scale: f64,
    ) -> Self {
        self.native_cursor_scale = if native_cursor_scale.is_finite() {
            native_cursor_scale.clamp(1.0, 3.0)
        } else {
            1.0
        };
        let Some(pointer_location) = pointer_location else {
            return self;
        };

        let cursor_anchor = pointer_location.to_i32_round::<i32>();

        match status {
            CursorImageStatus::Hidden => {}
            CursorImageStatus::Named(_) => {
                self.native_cursor_location = Some(cursor::default_cursor_origin(pointer_location));
            }
            CursorImageStatus::Surface(surface) => {
                if !surface.alive() {
                    return self;
                }

                let hotspot = with_states(surface, |states| {
                    states.data_map.get::<CursorImageSurfaceData>().map(|data| {
                        data.lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .hotspot
                    })
                });

                let Some(hotspot) = hotspot else {
                    return self;
                };

                let location: Point<i32, Logical> =
                    (pointer_location - hotspot.to_f64()).to_i32_round();
                self.cursor_surface = Some((surface.clone(), location, cursor_anchor));
            }
        }

        self
    }

    /// Returns the compositor-owned cursor origin in global logical coordinates.
    ///
    /// Client-provided cursor surfaces are kept in the regular Wayland surface tree.
    pub fn native_cursor_location(&self) -> Option<Point<i32, Logical>> {
        self.native_cursor_location
    }

    /// Returns the current scale for the compositor-owned cursor image.
    pub fn native_cursor_scale(&self) -> f64 {
        self.native_cursor_scale
    }

    /// Returns the toplevel surfaces in this scene.
    pub fn toplevels(&self) -> impl Iterator<Item = &'a ToplevelSurface> + '_ {
        self.toplevels.iter().map(|toplevel| toplevel.surface)
    }

    /// Returns the topmost client surface under a global logical pointer position.
    ///
    /// Smithay applies committed surface geometry and input regions, including
    /// subsurface stacking. The returned origin is in global logical coordinates.
    pub fn surface_under(
        &self,
        point: Point<f64, Logical>,
    ) -> Option<(WlSurface, Point<i32, Logical>, &'a ToplevelSurface)> {
        for toplevel in self.toplevels.iter().rev() {
            if let Some((surface, location)) = under_from_surface_tree(
                toplevel.surface.wl_surface(),
                point,
                toplevel.location,
                WindowSurfaceType::TOPLEVEL | WindowSurfaceType::SUBSURFACE,
            ) {
                return Some((surface, location, toplevel.surface));
            }
        }

        None
    }

    /// Returns the topmost toplevel containing a client surface at this point.
    pub fn toplevel_under(&self, point: Point<f64, Logical>) -> Option<&'a ToplevelSurface> {
        self.surface_under(point).map(|(_, _, toplevel)| toplevel)
    }

    /// Imports mapped toplevel surface trees as Smithay render elements.
    ///
    /// Window and output positions are stored in logical coordinates. Elements
    /// are positioned relative to the output origin and converted to physical
    /// pixels using the target output's scale factor.
    pub fn render_elements<R>(
        &self,
        renderer: &mut R,
        scale: f64,
        output_origin: (i32, i32),
    ) -> Vec<WaylandSurfaceRenderElement<R>>
    where
        R: Renderer + ImportAll,
        R::TextureId: Clone + 'static,
    {
        let mut elements = Vec::new();

        for toplevel in &self.toplevels {
            let location: Point<i32, Physical> = Point::<f64, Logical>::from((
                f64::from(toplevel.location.x) - f64::from(output_origin.0),
                f64::from(toplevel.location.y) - f64::from(output_origin.1),
            ))
            .to_physical(scale)
            .to_i32_round();

            elements.extend(render_elements_from_surface_tree(
                renderer,
                toplevel.surface.wl_surface(),
                location,
                scale,
                1.0,
                Kind::Unspecified,
            ));
        }

        elements
    }

    /// Returns full identities for every surface in every mapped surface tree.
    ///
    /// Unlike shm_snapshots, this includes surfaces that currently have no
    /// attached SHM buffer and therefore cannot produce a render snapshot.
    pub fn surface_ids(&self) -> Vec<smithay::reexports::wayland_server::backend::ObjectId> {
        let mut surface_ids = self
            .toplevels
            .iter()
            .flat_map(|toplevel| collect_surface_tree_ids(toplevel.surface.wl_surface()))
            .collect::<Vec<_>>();

        if let Some((surface, _, _)) = &self.cursor_surface {
            surface_ids.extend(collect_surface_tree_ids(surface));
        }

        surface_ids
    }

    /// Returns owned SHM snapshots for every mapped surface in tree render order.
    ///
    /// Cursor snapshots are appended last so the Vulkan compositor draws them
    /// above application windows. Output clipping naturally handles monitors
    /// that do not intersect the cursor rectangle.
    pub fn shm_snapshots(&self) -> Vec<SurfaceTreeSnapshot> {
        let mut snapshots = self
            .toplevels
            .iter()
            .flat_map(|toplevel| {
                collect_surface_tree_snapshots(toplevel.surface.wl_surface(), toplevel.location)
            })
            .collect::<Vec<_>>();

        if let Some((surface, location, cursor_anchor)) = &self.cursor_surface {
            let mut cursor_snapshots = collect_surface_tree_snapshots(surface, *location);
            for snapshot in &mut cursor_snapshots {
                snapshot.cursor_scale = self.native_cursor_scale;
                snapshot.cursor_anchor = Some(*cursor_anchor);
            }
            snapshots.extend(cursor_snapshots);
        }

        snapshots
    }

    /// Returns the number of mapped toplevels in this scene.
    pub fn len(&self) -> usize {
        self.toplevels.len()
    }

    /// Returns whether this scene contains no mapped toplevels.
    pub fn is_empty(&self) -> bool {
        self.toplevels.is_empty()
    }
}
