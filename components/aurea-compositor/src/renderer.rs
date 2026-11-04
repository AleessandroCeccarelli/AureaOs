// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Backend-independent rendering interface.

use std::error::Error;

use crate::render::RenderScene;

/// A backend capable of rendering an AureaOS scene.
pub trait RenderBackend {
    /// Error returned when rendering fails.
    type Error: Error + Send + Sync + 'static;

    /// Renders the supplied scene.
    fn render_scene(
        &mut self,
        scene: &RenderScene<'_>,
    ) -> Result<(), Self::Error>;
}
