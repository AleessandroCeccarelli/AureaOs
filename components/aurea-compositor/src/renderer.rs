// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Backend-independent rendering interface and GPU selection policy.

use std::error::Error;

use crate::render::RenderScene;

/// Preference for choosing a graphics device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPreference {
    /// Prefer the integrated GPU to reduce power consumption.
    PowerSaving,

    /// Prefer the discrete GPU for demanding workloads.
    Performance,

    /// Let the compositor choose based on system policy.
    Automatic,
}

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
