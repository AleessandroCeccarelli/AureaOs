// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Backend-independent rendering interface and GPU selection policy.

use std::error::Error;

use crate::{
    output::{DisplayOutput, OutputId},
    render::RenderScene,
};

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

/// Backend-independent description of a render target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderTarget {
    output_id: OutputId,
    width: u32,
    height: u32,
    refresh_millihz: u32,
    scale: u32,
}

impl RenderTarget {
    /// Creates a render target from an enabled display output.
    ///
    /// Disabled outputs cannot be used as render targets.
    pub fn from_output(output: &DisplayOutput) -> Option<Self> {
        if !output.is_enabled() {
            return None;
        }

        let mode = output.current_mode();

        Some(Self {
            output_id: output.id(),
            width: mode.width(),
            height: mode.height(),
            refresh_millihz: mode.refresh_millihz(),
            scale: output.scale(),
        })
    }

    /// Stable identifier of the display output.
    pub fn output_id(self) -> OutputId {
        self.output_id
    }

    /// Physical target width in pixels.
    pub fn width(self) -> u32 {
        self.width
    }

    /// Physical target height in pixels.
    pub fn height(self) -> u32 {
        self.height
    }

    /// Target refresh rate in millihertz.
    pub fn refresh_millihz(self) -> u32 {
        self.refresh_millihz
    }

    /// Integer scale factor configured for the output.
    pub fn scale(self) -> u32 {
        self.scale
    }
}

/// A backend capable of rendering an AureaOS scene to a target.
pub trait RenderBackend {
    /// Error returned when rendering fails.
    type Error: Error + Send + Sync + 'static;

    /// Renders and presents a scene on the specified output.
    ///
    /// The backend is responsible for translating the scene into its
    /// graphics API and presenting the resulting frame on the target.
    fn render_frame(
        &mut self,
        target: &RenderTarget,
        scene: &RenderScene<'_>,
    ) -> Result<(), Self::Error>;
}
