// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Small compositor-owned cursor bitmaps and logical hotspot metadata.

use crate::render::shm_buffer::{ShmBufferError, ShmBufferSnapshot};
use smithay::utils::{Logical, Point};

/// Native AureaOS arrow cursor dimensions in logical pixels.
pub const DEFAULT_CURSOR_WIDTH: u32 = 32;
pub const DEFAULT_CURSOR_HEIGHT: u32 = 32;

/// Hotspot of the default arrow, measured from its top-left pixel.
pub const DEFAULT_CURSOR_HOTSPOT: (i32, i32) = (5, 2);

/// Converts the global pointer position to the native arrow's top-left origin.
///
/// The hotspot is subtracted in logical coordinates before rounding to pixels.
pub(crate) fn default_cursor_origin(pointer_location: Point<f64, Logical>) -> Point<i32, Logical> {
    let hotspot = Point::<i32, Logical>::from(DEFAULT_CURSOR_HOTSPOT);
    (pointer_location - hotspot.to_f64()).to_i32_round()
}

/// Creates the built-in high-contrast arrow as tightly packed RGBA8 pixels.
///
/// The shape is derived from the AureaOS source cursor silhouette. This small
/// rasterizer intentionally handles only this static built-in glyph; SVG
/// parsing and animated theme assets remain a separate asset-build concern.
pub fn default_cursor_snapshot() -> Result<ShmBufferSnapshot, ShmBufferError> {
    let outline = [
        (5.0_f32, 2.0_f32),
        (5.0, 24.5),
        (10.5, 19.0),
        (14.5, 27.5),
        (18.5, 25.5),
        (14.5, 17.0),
        (22.5, 17.0),
    ];
    let mut pixels = vec![0_u8; (DEFAULT_CURSOR_WIDTH * DEFAULT_CURSOR_HEIGHT * 4) as usize];

    for y in 0..DEFAULT_CURSOR_HEIGHT {
        for x in 0..DEFAULT_CURSOR_WIDTH {
            let point = (x as f32 + 0.5, y as f32 + 0.5);
            let inside = point_in_polygon(point, &outline);
            let border = outline
                .iter()
                .zip(outline.iter().cycle().skip(1))
                .take(outline.len())
                .any(|(start, end)| distance_to_segment(point, *start, *end) <= 1.15);

            if inside || border {
                let offset = ((y * DEFAULT_CURSOR_WIDTH + x) * 4) as usize;
                let color = if border {
                    [24, 35, 51, 255]
                } else {
                    [255, 255, 255, 255]
                };
                pixels[offset..offset + 4].copy_from_slice(&color);
            }
        }
    }

    ShmBufferSnapshot::from_rgba8(DEFAULT_CURSOR_WIDTH, DEFAULT_CURSOR_HEIGHT, pixels)
}

fn point_in_polygon(point: (f32, f32), polygon: &[(f32, f32)]) -> bool {
    let mut inside = false;
    let mut previous = polygon.len() - 1;

    for current in 0..polygon.len() {
        let (x1, y1) = polygon[current];
        let (x2, y2) = polygon[previous];
        let crosses = (y1 > point.1) != (y2 > point.1)
            && point.0 < (x2 - x1) * (point.1 - y1) / (y2 - y1) + x1;
        if crosses {
            inside = !inside;
        }
        previous = current;
    }

    inside
}

fn distance_to_segment(point: (f32, f32), start: (f32, f32), end: (f32, f32)) -> f32 {
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let length_squared = dx * dx + dy * dy;
    if length_squared == 0.0 {
        return ((point.0 - start.0).powi(2) + (point.1 - start.1).powi(2)).sqrt();
    }

    let projection =
        (((point.0 - start.0) * dx + (point.1 - start.1) * dy) / length_squared).clamp(0.0, 1.0);
    let nearest = (start.0 + projection * dx, start.1 + projection * dy);
    ((point.0 - nearest.0).powi(2) + (point.1 - nearest.1).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_cursor_origin_applies_hotspot_before_rounding() {
        let pointer = Point::<f64, Logical>::from((10.6, 20.2));
        let origin = default_cursor_origin(pointer);

        assert_eq!((origin.x, origin.y), (6, 18));
    }

    #[test]
    fn default_cursor_has_expected_dimensions_and_hotspot() {
        let cursor = default_cursor_snapshot().unwrap();

        assert_eq!(cursor.width(), DEFAULT_CURSOR_WIDTH);
        assert_eq!(cursor.height(), DEFAULT_CURSOR_HEIGHT);
        assert_eq!(DEFAULT_CURSOR_HOTSPOT, (5, 2));
    }

    #[test]
    fn cursor_pixels_are_transparent_outside_the_arrow() {
        let cursor = default_cursor_snapshot().unwrap();
        let pixels = cursor.pixels();

        assert_eq!(&pixels[0..4], &[0, 0, 0, 0]);
        let interior = ((8 * DEFAULT_CURSOR_WIDTH + 8) * 4) as usize;
        assert_eq!(&pixels[interior..interior + 4], &[255, 255, 255, 255]);
    }

    #[test]
    fn cursor_outline_uses_a_dark_high_contrast_edge() {
        let cursor = default_cursor_snapshot().unwrap();
        let pixels = cursor.pixels();
        let outline = ((10 * DEFAULT_CURSOR_WIDTH + 5) * 4) as usize;

        assert_eq!(&pixels[outline..outline + 4], &[24, 35, 51, 255]);
    }
}
