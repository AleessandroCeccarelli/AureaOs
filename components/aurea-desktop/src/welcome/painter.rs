// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Pixel-level drawing primitives for the temporary Welcome surface.

const WIDTH: usize = 720;
const HEIGHT: usize = 440;

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

