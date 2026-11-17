// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Pixel-level drawing primitives for the temporary Welcome surface.

use super::{HEIGHT, WIDTH};

pub(super) fn paint_welcome(pixels: &mut [u8]) {
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let offset = (y * WIDTH + x) * 4;
            let (r, g, b) = if y < 76 { (18, 39, 75) } else { (14, 22, 37) };
            pixels[offset..offset + 4].copy_from_slice(&[b, g, r, 255]);
        }
    }

    // Geometric Aurea mark and a restrained welcome layout.
    fill_rect(pixels, 54, 128, 92, 92, [220, 170, 55, 255]);
    fill_rect(pixels, 77, 150, 46, 48, [14, 22, 37, 255]);
    fill_rect(pixels, 86, 160, 28, 10, [220, 170, 55, 255]);
    fill_rect(pixels, 54, 256, 612, 2, [55, 83, 125, 255]);
    fill_rect(pixels, 54, 294, 310, 14, [226, 233, 244, 255]);
    fill_rect(pixels, 54, 324, 420, 8, [116, 145, 181, 255]);
}

fn fill_rect(pixels: &mut [u8], x: usize, y: usize, width: usize, height: usize, color: [u8; 4]) {
    for row in y..(y + height).min(HEIGHT) {
        for column in x..(x + width).min(WIDTH) {
            let offset = (row * WIDTH + column) * 4;
            pixels[offset..offset + 4].copy_from_slice(&color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HEIGHT, WIDTH, paint_welcome};

    #[test]
    fn welcome_surface_has_expected_background_and_header() {
        let mut pixels = vec![0; WIDTH * HEIGHT * 4];
        paint_welcome(&mut pixels);

        let pixel = |x: usize, y: usize| {
            let offset = (y * WIDTH + x) * 4;
            &pixels[offset..offset + 4]
        };

        assert_eq!(pixel(0, 0), [75, 39, 18, 255]);
        assert_eq!(pixel(0, 76), [37, 22, 14, 255]);
    }

    #[test]
    fn welcome_surface_draws_the_aurea_mark_and_content_guides() {
        let mut pixels = vec![0; WIDTH * HEIGHT * 4];
        paint_welcome(&mut pixels);

        let pixel = |x: usize, y: usize| {
            let offset = (y * WIDTH + x) * 4;
            &pixels[offset..offset + 4]
        };

        assert_eq!(pixel(54, 128), [55, 170, 220, 255]);
        assert_eq!(pixel(77, 150), [37, 22, 14, 255]);
        assert_eq!(pixel(54, 256), [125, 83, 55, 255]);
        assert_eq!(pixel(54, 294), [244, 233, 226, 255]);
    }
}
