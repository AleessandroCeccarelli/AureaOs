// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Small built-in 5x7 uppercase typeface for the first native Welcome surface.

use super::{HEIGHT, WIDTH};

const fn glyph(character: u8) -> [u8; 7] {
    match character {
        b'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        b'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        b'C' => [
            0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111,
        ],
        b'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        b'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        b'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        b'G' => [
            0b01111, 0b10000, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111,
        ],
        b'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        b'I' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
        ],
        b'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        b'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        b'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        b'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        b'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        b'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        b'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        b'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        b'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        b'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        b'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ],
        b'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010,
        ],
        b'Y' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        _ => [0; 7],
    }
}

/// Draws uppercase text using an integer scale and opaque RGBA color.
pub(super) fn draw_text(
    pixels: &mut [u8],
    mut x: usize,
    y: usize,
    text: &str,
    scale: usize,
    color: [u8; 4],
) {
    for character in text.bytes() {
        if character == b' ' {
            x += 4 * scale;
            continue;
        }

        let rows = glyph(character.to_ascii_uppercase());
        for (row, bits) in rows.into_iter().enumerate() {
            for column in 0..5 {
                if bits & (1 << (4 - column)) != 0 {
                    fill_block(pixels, x + column * scale, y + row * scale, scale, color);
                }
            }
        }
        x += 6 * scale;
    }
}

fn fill_block(pixels: &mut [u8], x: usize, y: usize, size: usize, color: [u8; 4]) {
    for row in y..(y + size).min(HEIGHT) {
        for column in x..(x + size).min(WIDTH) {
            let offset = (row * WIDTH + column) * 4;
            pixels[offset..offset + 4].copy_from_slice(&[color[2], color[1], color[0], color[3]]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HEIGHT, WIDTH, draw_text};

    #[test]
    fn draws_non_empty_glyph_pixels() {
        let mut pixels = vec![0; WIDTH * HEIGHT * 4];
        draw_text(&mut pixels, 8, 8, "AUREAOS", 2, [255, 255, 255, 255]);
        assert!(
            pixels
                .chunks_exact(4)
                .any(|pixel| pixel == [255, 255, 255, 255])
        );
    }

    #[test]
    fn ignores_unknown_glyphs_without_writing_outside_the_surface() {
        let mut pixels = vec![0; WIDTH * HEIGHT * 4];
        draw_text(&mut pixels, 710, 430, "?", 4, [255, 255, 255, 255]);
        assert!(pixels.iter().all(|channel| *channel == 0));
    }
}
