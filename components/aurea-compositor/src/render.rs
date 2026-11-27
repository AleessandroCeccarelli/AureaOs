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
#[cfg(feature = "drm")]
use std::collections::HashMap;

use aurea_desktop_api::{desktop::DesktopContextMenuLayout, desktop_interaction::DesktopPanelKind};

use crate::desktop::{DesktopItemsSnapshot, DesktopSnapshot, DesktopTheme};

use smithay::{
    backend::renderer::{
        ImportAll, Renderer,
        element::{
            Kind,
            surface::{WaylandSurfaceRenderElement, render_elements_from_surface_tree},
        },
    },
    desktop::{PopupManager, WindowSurfaceType, utils::under_from_surface_tree},
    input::pointer::{CursorImageStatus, CursorImageSurfaceData},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{IsAlive, Logical, Physical, Point},
    wayland::{compositor::with_states, shell::xdg::ToplevelSurface},
};

use crate::{input::cursor_trail::CursorTrailSample, window_manager::WindowManager};

/// Immutable desktop, context-menu, mapped-surface and pointer state for one rendering pass.
pub struct RenderScene<'a> {
    desktop: DesktopSnapshot,
    desktop_items: DesktopItemsSnapshot,
    context_menu: Option<DesktopContextMenuLayout>,
    active_panel: Option<DesktopPanelKind>,
    context_menu_theme: DesktopTheme,
    toplevels: Vec<RenderToplevel<'a>>,
    cursor_surface: Option<(WlSurface, Point<i32, Logical>, Point<i32, Logical>)>,
    native_cursor_location: Option<Point<i32, Logical>>,
    native_cursor_pointer_location: Option<Point<f64, Logical>>,
    native_cursor_asset: Option<&'static str>,
    native_cursor_scale: f64,
    cursor_trail: Vec<CursorTrailSample>,
}

struct RenderToplevel<'a> {
    surface: &'a ToplevelSurface,
    location: Point<i32, Logical>,
}

/// One renderable Wayland surface buffer, preserving the scene's stacking order.
#[cfg(feature = "drm")]
pub enum OrderedSurfaceBufferSnapshot {
    /// CPU-readable shared-memory content uploaded by the compositor.
    Shm(SurfaceTreeSnapshot),
    /// Client DMA-BUF content requiring device import and producer synchronization.
    Dmabuf(surface_tree::SurfaceTreeDmabufSnapshot),
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
            context_menu: None,
            active_panel: None,
            context_menu_theme: DesktopTheme::Light,
            toplevels: window_manager
                .mapped_toplevels_with_location()
                .map(|(surface, location)| RenderToplevel { surface, location })
                .collect(),
            cursor_surface: None,
            native_cursor_location: None,
            native_cursor_pointer_location: None,
            native_cursor_asset: None,
            native_cursor_scale: 1.0,
            cursor_trail: Vec::new(),
        }
    }

    /// Replaces the desktop state captured for this render frame.
    pub fn with_desktop(mut self, desktop: DesktopSnapshot) -> Self {
        self.desktop = desktop;
        self
    }

    /// Returns the immutable desktop state captured for this frame.
    pub fn desktop(&self) -> &DesktopSnapshot {
        &self.desktop
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

    /// Adds the compositor-owned desktop context-menu geometry for this frame.
    pub fn with_context_menu(
        mut self,
        menu: Option<DesktopContextMenuLayout>,
        theme: DesktopTheme,
    ) -> Self {
        self.context_menu = menu;
        self.context_menu_theme = theme;
        self
    }

    /// Adds the active semantic desktop panel to this immutable frame snapshot.
    pub fn with_active_panel(mut self, panel: Option<DesktopPanelKind>) -> Self {
        self.active_panel = panel;
        self
    }

    /// Returns the active semantic panel captured for this frame, if any.
    pub fn active_panel(&self) -> Option<DesktopPanelKind> {
        self.active_panel
    }

    /// Returns the context-menu geometry captured for this frame, if open.
    pub fn context_menu(&self) -> Option<&DesktopContextMenuLayout> {
        self.context_menu.as_ref()
    }

    /// Returns the appearance variant captured for the context menu.
    pub fn context_menu_theme(&self) -> DesktopTheme {
        self.context_menu_theme
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
            CursorImageStatus::Named(icon) => {
                self.native_cursor_location = Some(cursor::default_cursor_origin(pointer_location));
                self.native_cursor_pointer_location = Some(pointer_location);
                self.native_cursor_asset =
                    Some(crate::input::cursor_theme::asset_name_for_icon(icon.name()));
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

    /// Returns the global logical pointer position used to anchor a named cursor.
    pub fn native_cursor_pointer_location(&self) -> Option<Point<f64, Logical>> {
        self.native_cursor_pointer_location
    }

    /// Returns the canonical AureaOS asset selected for a named cursor.
    pub fn native_cursor_asset(&self) -> Option<&'static str> {
        self.native_cursor_asset
    }

    /// Returns the current scale for the compositor-owned cursor image.
    pub fn native_cursor_scale(&self) -> f64 {
        self.native_cursor_scale
    }

    /// Adds the short-lived historical positions used by the native cursor trail.
    pub fn with_cursor_trail(mut self, samples: Vec<CursorTrailSample>) -> Self {
        self.cursor_trail = samples;
        self
    }

    /// Returns newest-first cursor trail samples for this render frame.
    pub fn cursor_trail(&self) -> &[CursorTrailSample] {
        &self.cursor_trail
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
            let root = toplevel.surface.wl_surface();
            let popups = PopupManager::popups_for_surface(root).collect::<Vec<_>>();

            // Smithay yields nested children before their parents; preserve that
            // order here so the deepest/topmost popup wins hit testing.
            for (popup, popup_offset) in popups {
                let popup_origin = toplevel.location + popup_offset - popup.geometry().loc;
                if let Some((surface, location)) = under_from_surface_tree(
                    popup.wl_surface(),
                    point,
                    popup_origin,
                    WindowSurfaceType::POPUP | WindowSurfaceType::SUBSURFACE,
                ) {
                    return Some((surface, location, toplevel.surface));
                }
            }

            if let Some((surface, location)) = under_from_surface_tree(
                root,
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

            let root = toplevel.surface.wl_surface();
            elements.extend(render_elements_from_surface_tree(
                renderer,
                root,
                location,
                scale,
                1.0,
                Kind::Unspecified,
            ));

            for (popup, popup_offset) in PopupManager::popups_for_surface(root)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
            {
                let popup_origin = toplevel.location + popup_offset - popup.geometry().loc;
                let popup_location: Point<i32, Physical> = Point::<f64, Logical>::from((
                    f64::from(popup_origin.x) - f64::from(output_origin.0),
                    f64::from(popup_origin.y) - f64::from(output_origin.1),
                ))
                .to_physical(scale)
                .to_i32_round();

                elements.extend(render_elements_from_surface_tree(
                    renderer,
                    popup.wl_surface(),
                    popup_location,
                    scale,
                    1.0,
                    Kind::Unspecified,
                ));
            }
        }

        elements
    }

    /// Returns full identities for every surface in every mapped surface tree.
    ///
    /// Unlike shm_snapshots, this includes surfaces that currently have no
    /// attached SHM buffer and therefore cannot produce a render snapshot.
    pub fn surface_ids(&self) -> Vec<smithay::reexports::wayland_server::backend::ObjectId> {
        let mut surface_ids = Vec::new();
        for toplevel in &self.toplevels {
            let root = toplevel.surface.wl_surface();
            surface_ids.extend(collect_surface_tree_ids(root));
            for (popup, _) in PopupManager::popups_for_surface(root) {
                surface_ids.extend(collect_surface_tree_ids(popup.wl_surface()));
            }
        }

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
        let mut snapshots = Vec::new();
        for toplevel in &self.toplevels {
            let root = toplevel.surface.wl_surface();
            // The Vulkan compositor draws snapshots in vector order. Keep the
            // parent tree first so every popup is composited above its owner.
            snapshots.extend(collect_surface_tree_snapshots(root, toplevel.location));
            for (popup, popup_offset) in PopupManager::popups_for_surface(root) {
                let popup_origin = toplevel.location + popup_offset - popup.geometry().loc;
                snapshots.extend(collect_surface_tree_snapshots(
                    popup.wl_surface(),
                    popup_origin,
                ));
            }
        }

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

    /// Returns retained DMA-BUF snapshots in the same order as SHM surfaces.
    ///
    /// This is the renderer-facing acquisition boundary. The Vulkan importer
    /// must still validate device-specific format/modifier support before use.
    #[cfg(feature = "drm")]
    pub fn dmabuf_snapshots(&self) -> Vec<surface_tree::SurfaceTreeDmabufSnapshot> {
        let mut snapshots = Vec::new();
        for toplevel in &self.toplevels {
            let root = toplevel.surface.wl_surface();
            snapshots.extend(surface_tree::collect_surface_tree_dmabuf_snapshots(
                root,
                toplevel.location,
            ));
            for (popup, popup_offset) in PopupManager::popups_for_surface(root) {
                let popup_origin = toplevel.location + popup_offset - popup.geometry().loc;
                snapshots.extend(surface_tree::collect_surface_tree_dmabuf_snapshots(
                    popup.wl_surface(),
                    popup_origin,
                ));
            }
        }

        if let Some((surface, location, _)) = &self.cursor_surface {
            snapshots.extend(surface_tree::collect_surface_tree_dmabuf_snapshots(
                surface, *location,
            ));
        }

        snapshots
    }

    /// Returns SHM and DMA-BUF snapshots in one authoritative surface stacking order.
    ///
    /// The renderer must consume this sequence rather than concatenating the
    /// separate SHM and DMA-BUF lists: doing so would move every DMA-BUF surface
    /// above every SHM surface and visibly break Wayland subsurface stacking.
    #[cfg(feature = "drm")]
    pub fn ordered_buffer_snapshots(&self) -> Vec<OrderedSurfaceBufferSnapshot> {
        let mut shm = self.shm_snapshots()
            .into_iter()
            .map(|snapshot| (snapshot.surface_id.clone(), snapshot))
            .collect::<HashMap<_, _>>();
        let mut dmabuf = self.dmabuf_snapshots()
            .into_iter()
            .map(|snapshot| (snapshot.surface_id.clone(), snapshot))
            .collect::<HashMap<_, _>>();

        self.surface_ids()
            .into_iter()
            .filter_map(|surface_id| {
                if let Some(snapshot) = shm.remove(&surface_id) {
                    Some(OrderedSurfaceBufferSnapshot::Shm(snapshot))
                } else {
                    dmabuf.remove(&surface_id)
                        .map(OrderedSurfaceBufferSnapshot::Dmabuf)
                }
            })
            .collect()
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

#[cfg(test)]
mod cursor_scene_tests {
    use super::*;
    use crate::window_manager::WindowManager;

    #[test]
    fn default_named_cursor_is_present_in_the_scene_at_the_pointer_location() {
        let windows = WindowManager::new();
        let pointer = Point::<f64, Logical>::from((640.0, 360.0));
        let status = CursorImageStatus::default_named();

        let scene =
            RenderScene::from_window_manager(&windows).with_cursor(&status, Some(pointer), 1.0);

        assert_eq!(
            scene.native_cursor_location(),
            Some(cursor::default_cursor_origin(pointer))
        );
        assert_eq!(scene.native_cursor_pointer_location(), Some(pointer));
        assert_eq!(scene.native_cursor_asset(), Some("default"));
    }

    #[test]
    fn named_cursor_selection_is_carried_into_the_render_scene() {
        let windows = WindowManager::new();
        let pointer = Point::<f64, Logical>::from((640.0, 360.0));
        let status = CursorImageStatus::Named(smithay::input::pointer::CursorIcon::Help);

        let scene =
            RenderScene::from_window_manager(&windows).with_cursor(&status, Some(pointer), 1.0);

        assert_eq!(scene.native_cursor_asset(), Some("help"));
    }

    #[test]
    fn missing_pointer_location_does_not_create_a_cursor_rectangle() {
        let windows = WindowManager::new();
        let status = CursorImageStatus::default_named();

        let scene = RenderScene::from_window_manager(&windows).with_cursor(&status, None, 1.0);

        assert_eq!(scene.native_cursor_location(), None);
    }

    #[test]
    fn explicitly_hidden_cursor_is_not_rendered() {
        let windows = WindowManager::new();
        let pointer = Point::<f64, Logical>::from((640.0, 360.0));

        let scene = RenderScene::from_window_manager(&windows).with_cursor(
            &CursorImageStatus::Hidden,
            Some(pointer),
            1.0,
        );

        assert_eq!(scene.native_cursor_location(), None);
        assert_eq!(scene.native_cursor_pointer_location(), None);
        assert_eq!(scene.native_cursor_asset(), None);
    }
}
