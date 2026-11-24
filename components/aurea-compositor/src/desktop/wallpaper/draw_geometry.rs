// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Output-local Vulkan draw geometry derived from wallpaper fit calculations.

use super::{WallpaperLayout, WallpaperPixelRect};

/// Normalized source-image coordinates for one clipped wallpaper draw.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WallpaperUvRect {
    min: [f32; 2],
    max: [f32; 2],
}

impl WallpaperUvRect {
    pub const fn min(self) -> [f32; 2] {
        self.min
    }

    pub const fn max(self) -> [f32; 2] {
        self.max
    }
}

/// Destination rectangle and source UV range for one physical output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WallpaperDrawGeometry {
    destination: WallpaperPixelRect,
    uv: WallpaperUvRect,
}

impl WallpaperDrawGeometry {
    /// Clips the destination to the output and adjusts UVs by the same ratio.
    ///
    /// The viewport is the clipped destination in output-local physical pixels.
    /// Callers must use the returned destination as both viewport and scissor;
    /// this avoids negative Vulkan scissor offsets while preserving the correct
    /// source crop for Center and any future oversized layout.
    pub fn for_output(
        layout: WallpaperLayout,
        source_width: u32,
        source_height: u32,
        output_width: u32,
        output_height: u32,
    ) -> Option<Self> {
        if source_width == 0 || source_height == 0 {
            return None;
        }

        let source = layout.source();
        let destination = layout.destination();
        let output = WallpaperPixelRect::new(0, 0, output_width, output_height)?;
        let clipped = destination.intersection(output)?;

        let left_fraction = (clipped.x() - destination.x()) as f64
            / f64::from(destination.width());
        let top_fraction = (clipped.y() - destination.y()) as f64
            / f64::from(destination.height());
        let right_fraction = (clipped.x() + i64::from(clipped.width())
            - destination.x()) as f64
            / f64::from(destination.width());
        let bottom_fraction = (clipped.y() + i64::from(clipped.height())
            - destination.y()) as f64
            / f64::from(destination.height());

        let source_left = source.x() as f64;
        let source_top = source.y() as f64;
        let source_width_f = f64::from(source.width());
        let source_height_f = f64::from(source.height());

        let uv = WallpaperUvRect {
            min: [
                ((source_left + source_width_f * left_fraction) / f64::from(source_width)) as f32,
                ((source_top + source_height_f * top_fraction) / f64::from(source_height)) as f32,
            ],
            max: [
                ((source_left + source_width_f * right_fraction) / f64::from(source_width)) as f32,
                ((source_top + source_height_f * bottom_fraction) / f64::from(source_height)) as f32,
            ],
        };

        Some(Self {
            destination: clipped,
            uv,
        })
    }

    pub const fn destination(self) -> WallpaperPixelRect {
        self.destination
    }

    pub const fn uv(self) -> WallpaperUvRect {
        self.uv
    }
}

#[cfg(test)]
mod tests {
    use super::WallpaperDrawGeometry;
    use crate::desktop::{WallpaperFitMode, WallpaperLayout};

    fn geometry(
        source: (u32, u32),
        output: (u32, u32),
        mode: WallpaperFitMode,
    ) -> WallpaperDrawGeometry {
        let layout = WallpaperLayout::calculate(
            source.0, source.1, output.0, output.1, mode,
        )
        .unwrap();
        WallpaperDrawGeometry::for_output(
            layout, source.0, source.1, output.0, output.1,
        )
        .unwrap()
    }

    #[test]
    fn fill_maps_the_center_crop_to_the_complete_output() {
        let draw = geometry((4000, 2000), (1920, 1080), WallpaperFitMode::Fill);

        assert_eq!(draw.destination().x(), 0);
        assert_eq!(draw.destination().y(), 0);
        assert_eq!(draw.destination().width(), 1920);
        assert_eq!(draw.destination().height(), 1080);
        assert!((draw.uv().min()[0] - 0.0555).abs() < 0.001);
        assert!((draw.uv().max()[0] - 0.9445).abs() < 0.001);
        assert_eq!(draw.uv().min()[1], 0.0);
        assert_eq!(draw.uv().max()[1], 1.0);
    }

    #[test]
    fn fit_keeps_full_source_and_returns_letterboxed_destination() {
        let draw = geometry((4000, 2000), (1920, 1080), WallpaperFitMode::Fit);

        assert_eq!(draw.destination().y(), 60);
        assert_eq!(draw.destination().height(), 960);
        assert_eq!(draw.uv().min(), [0.0, 0.0]);
        assert_eq!(draw.uv().max(), [1.0, 1.0]);
    }

    #[test]
    fn stretch_uses_full_source_and_output() {
        let draw = geometry((800, 600), (1920, 1080), WallpaperFitMode::Stretch);

        assert_eq!(draw.destination().width(), 1920);
        assert_eq!(draw.destination().height(), 1080);
        assert_eq!(draw.uv().min(), [0.0, 0.0]);
        assert_eq!(draw.uv().max(), [1.0, 1.0]);
    }

    #[test]
    fn center_clipping_adjusts_uvs_without_negative_destination() {
        let draw = geometry((2560, 1440), (1920, 1080), WallpaperFitMode::Center);

        assert_eq!(draw.destination().x(), 0);
        assert_eq!(draw.destination().y(), 0);
        assert_eq!(draw.destination().width(), 1920);
        assert_eq!(draw.destination().height(), 1080);
        assert!((draw.uv().min()[0] - 0.125).abs() < f32::EPSILON);
        assert!((draw.uv().max()[0] - 0.875).abs() < f32::EPSILON);
        assert!((draw.uv().min()[1] - 0.125).abs() < f32::EPSILON);
        assert!((draw.uv().max()[1] - 0.875).abs() < f32::EPSILON);
    }

    #[test]
    fn fit_returns_none_when_destination_does_not_intersect_output() {
        let layout = WallpaperLayout::calculate(
            100, 100, 1920, 1080, WallpaperFitMode::Fit,
        )
        .unwrap();
        assert!(WallpaperDrawGeometry::for_output(layout, 100, 100, 0, 1080).is_none());
    }
}
