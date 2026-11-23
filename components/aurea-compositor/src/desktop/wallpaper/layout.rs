// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Output-local placement policy for desktop wallpaper images.

/// User-facing way to fit a wallpaper into one display output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallpaperFitMode {
    /// Preserve aspect ratio and crop the excess image area.
    Fill,
    /// Preserve aspect ratio and show the complete image with empty margins.
    Fit,
    /// Resize the image independently on both axes.
    Stretch,
    /// Keep the image at its native pixel dimensions and center it.
    Center,
}

/// Integer pixel rectangle. Coordinates may be negative for centered images
/// larger than the output; the output scissor clips those pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallpaperPixelRect {
    x: i64,
    y: i64,
    width: u32,
    height: u32,
}

impl WallpaperPixelRect {
    pub const fn x(self) -> i64 { self.x }
    pub const fn y(self) -> i64 { self.y }
    pub const fn width(self) -> u32 { self.width }
    pub const fn height(self) -> u32 { self.height }
}

/// Source crop and destination placement for one output.
///
/// The source rectangle is expressed in source-image pixels. The destination
/// rectangle is expressed in output-local physical pixels. Rendering must
/// apply an output-sized scissor so Center can safely extend beyond the output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallpaperLayout {
    source: WallpaperPixelRect,
    destination: WallpaperPixelRect,
}

impl WallpaperLayout {
    /// Calculates a layout without floating-point arithmetic or allocation.
    ///
    /// Returns None for zero-sized source images or outputs.
    pub fn calculate(
        source_width: u32,
        source_height: u32,
        output_width: u32,
        output_height: u32,
        mode: WallpaperFitMode,
    ) -> Option<Self> {
        if source_width == 0 || source_height == 0 || output_width == 0 || output_height == 0 {
            return None;
        }

        let source_full = WallpaperPixelRect {
            x: 0, y: 0, width: source_width, height: source_height,
        };
        let output_full = WallpaperPixelRect {
            x: 0, y: 0, width: output_width, height: output_height,
        };

        let (source, destination) = match mode {
            WallpaperFitMode::Stretch => (source_full, output_full),
            WallpaperFitMode::Fill => (
                centered_cover_crop(source_width, source_height, output_width, output_height),
                output_full,
            ),
            WallpaperFitMode::Fit => (
                source_full,
                centered_contain_rect(source_width, source_height, output_width, output_height),
            ),
            WallpaperFitMode::Center => (
                source_full,
                WallpaperPixelRect {
                    x: (i64::from(output_width) - i64::from(source_width)) / 2,
                    y: (i64::from(output_height) - i64::from(source_height)) / 2,
                    width: source_width,
                    height: source_height,
                },
            ),
        };

        Some(Self { source, destination })
    }

    pub const fn source(self) -> WallpaperPixelRect { self.source }
    pub const fn destination(self) -> WallpaperPixelRect { self.destination }
}

fn centered_cover_crop(
    source_width: u32,
    source_height: u32,
    output_width: u32,
    output_height: u32,
) -> WallpaperPixelRect {
    let source_ratio_cross = u64::from(source_width) * u64::from(output_height);
    let output_ratio_cross = u64::from(output_width) * u64::from(source_height);

    if source_ratio_cross > output_ratio_cross {
        let crop_width = (((u64::from(source_height) * u64::from(output_width))
            / u64::from(output_height)) as u32)
            .clamp(1, source_width);
        WallpaperPixelRect {
            x: i64::from((source_width - crop_width) / 2),
            y: 0,
            width: crop_width,
            height: source_height,
        }
    } else {
        let crop_height = (((u64::from(source_width) * u64::from(output_height))
            / u64::from(output_width)) as u32)
            .clamp(1, source_height);
        WallpaperPixelRect {
            x: 0,
            y: i64::from((source_height - crop_height) / 2),
            width: source_width,
            height: crop_height,
        }
    }
}

fn centered_contain_rect(
    source_width: u32,
    source_height: u32,
    output_width: u32,
    output_height: u32,
) -> WallpaperPixelRect {
    let source_ratio_cross = u64::from(source_width) * u64::from(output_height);
    let output_ratio_cross = u64::from(output_width) * u64::from(source_height);

    let (width, height) = if source_ratio_cross > output_ratio_cross {
        let height = ((u64::from(source_height) * u64::from(output_width))
            / u64::from(source_width)) as u32;
        (output_width, height.max(1))
    } else {
        let width = ((u64::from(source_width) * u64::from(output_height))
            / u64::from(source_height)) as u32;
        (width.max(1), output_height)
    };

    WallpaperPixelRect {
        x: (i64::from(output_width) - i64::from(width)) / 2,
        y: (i64::from(output_height) - i64::from(height)) / 2,
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::{WallpaperFitMode, WallpaperLayout};

    #[test]
    fn fill_crops_wide_source_to_output_aspect_ratio() {
        let layout =
            WallpaperLayout::calculate(4000, 2000, 1920, 1080, WallpaperFitMode::Fill).unwrap();

        assert_eq!(layout.source().x(), 222);
        assert_eq!(layout.source().y(), 0);
        assert_eq!(layout.source().width(), 3555);
        assert_eq!(layout.source().height(), 2000);
        assert_eq!(layout.destination().width(), 1920);
        assert_eq!(layout.destination().height(), 1080);
    }

    #[test]
    fn fit_keeps_the_complete_image_and_centers_it() {
        let layout =
            WallpaperLayout::calculate(4000, 2000, 1920, 1080, WallpaperFitMode::Fit).unwrap();

        assert_eq!(layout.source().width(), 4000);
        assert_eq!(layout.source().height(), 2000);
        assert_eq!(layout.destination().x(), 0);
        assert_eq!(layout.destination().y(), 60);
        assert_eq!(layout.destination().width(), 1920);
        assert_eq!(layout.destination().height(), 960);
    }

    #[test]
    fn stretch_uses_the_entire_source_and_output() {
        let layout =
            WallpaperLayout::calculate(800, 600, 1920, 1080, WallpaperFitMode::Stretch).unwrap();

        assert_eq!(layout.source().width(), 800);
        assert_eq!(layout.source().height(), 600);
        assert_eq!(layout.destination().x(), 0);
        assert_eq!(layout.destination().y(), 0);
        assert_eq!(layout.destination().width(), 1920);
        assert_eq!(layout.destination().height(), 1080);
    }

    #[test]
    fn center_keeps_native_size_and_can_extend_past_output_edges() {
        let layout =
            WallpaperLayout::calculate(2560, 1440, 1920, 1080, WallpaperFitMode::Center).unwrap();

        assert_eq!(layout.destination().x(), -320);
        assert_eq!(layout.destination().y(), -180);
        assert_eq!(layout.destination().width(), 2560);
        assert_eq!(layout.destination().height(), 1440);
    }

    #[test]
    fn zero_sized_images_or_outputs_are_rejected() {
        assert!(WallpaperLayout::calculate(0, 100, 1920, 1080, WallpaperFitMode::Fill).is_none());
        assert!(WallpaperLayout::calculate(100, 100, 0, 1080, WallpaperFitMode::Fit).is_none());
    }
}
