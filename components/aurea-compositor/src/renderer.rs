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
    fractional_scale: Option<crate::output::scale::ScaleFactor>,
    logical_origin: (i32, i32),
}

impl RenderTarget {
    /// Creates a render target from an enabled output and its logical position.
    ///
    /// The origin is supplied by the output layout. Keeping it on the target
    /// lets each backend translate global surface coordinates into coordinates
    /// local to this output, including multi-monitor arrangements.
    pub fn from_output(output: &DisplayOutput, logical_origin: (i32, i32)) -> Option<Self> {
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
            fractional_scale: output.fractional_scale(),
            logical_origin,
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

    /// Legacy integer scale factor configured for the output.
    pub fn scale(self) -> u32 {
        self.scale
    }

    /// Output origin in the compositor's global logical coordinate space.
    /// Optional exact fractional scale for this output.
    pub fn fractional_scale(self) -> Option<crate::output::scale::ScaleFactor> {
        self.fractional_scale
    }

    /// Effective scale for geometry calculations.
    pub fn effective_scale(self) -> f64 {
        self.fractional_scale.map_or(f64::from(self.scale), |scale| scale.as_f64())
    }

    pub fn logical_origin(self) -> (i32, i32) {
        self.logical_origin
    }
}

/// A backend capable of rendering an AureaOS scene to a target.
pub trait RenderBackend {
    /// Error returned when rendering fails.
    type Error: Error + Send + Sync + 'static;

    /// Renders and presents a scene on the specified output.
    ///
    /// The backend is responsible for translating the scene into its
    /// graphics API and presenting the resulting frame.
    fn render_frame(
        &mut self,
        target: &RenderTarget,
        scene: &RenderScene<'_>,
    ) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::OutputMode;

    fn output(enabled: bool) -> DisplayOutput {
        let mode = OutputMode::new(1920, 1080, 60_000).unwrap();
        let mut output = DisplayOutput::new(7, "DP-1", vec![mode], mode, 2).unwrap();
        output.set_enabled(enabled);
        output
    }

    #[test]
    fn render_target_preserves_output_layout_origin_and_scale() {
        let target = RenderTarget::from_output(&output(true), (-1920, 120)).unwrap();

        assert_eq!(target.output_id(), 7);
        assert_eq!(target.width(), 1920);
        assert_eq!(target.height(), 1080);
        assert_eq!(target.scale(), 2);
        assert_eq!(target.logical_origin(), (-1920, 120));
    }

    #[test]
    fn disabled_output_cannot_become_a_render_target() {
        assert!(RenderTarget::from_output(&output(false), (0, 0)).is_none());
    }
}
