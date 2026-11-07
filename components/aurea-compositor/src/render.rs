// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Collects mapped surfaces into a renderable scene.

use smithay::{
    backend::renderer::{
        ImportAll, Renderer,
        element::{
            Kind,
            surface::{WaylandSurfaceRenderElement, render_elements_from_surface_tree},
        },
    },
    utils::{Logical, Physical, Point},
    wayland::shell::xdg::ToplevelSurface,
};

use crate::window_manager::WindowManager;

/// A snapshot of the mapped toplevels for one rendering pass.
pub struct RenderScene<'a> {
    toplevels: Vec<RenderToplevel<'a>>,
}

struct RenderToplevel<'a> {
    surface: &'a ToplevelSurface,
    location: Point<i32, Logical>,
}

impl<'a> RenderScene<'a> {
    /// Builds a scene from the window manager's currently mapped toplevels.
    pub fn from_window_manager(window_manager: &'a WindowManager) -> Self {
        Self {
            toplevels: window_manager
                .mapped_toplevels_with_location()
                .map(|(surface, location)| RenderToplevel { surface, location })
                .collect(),
        }
    }

    /// Returns the toplevel surfaces in this scene.
    pub fn toplevels(&self) -> impl Iterator<Item = &'a ToplevelSurface> + '_ {
        self.toplevels.iter().map(|toplevel| toplevel.surface)
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

    /// Returns the number of surfaces collected for this scene.
    pub fn len(&self) -> usize {
        self.toplevels.len()
    }

    /// Returns whether this scene contains no mapped surfaces.
    pub fn is_empty(&self) -> bool {
        self.toplevels.is_empty()
    }
}
