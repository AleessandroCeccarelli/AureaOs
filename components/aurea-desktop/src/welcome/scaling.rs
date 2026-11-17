// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Scaling of the fixed Welcome artwork into a configured surface size.

use super::{HEIGHT, WIDTH, painter::paint_welcome};

pub(super) fn render_scaled(width: usize, height: usize) -> Vec<u8> {
    let mut source = vec![0; WIDTH * HEIGHT * 4];
    paint_welcome(&mut source);

    let mut target = vec![0; width * height * 4];
    for y in 0..height {
        let source_y = y * HEIGHT / height;
        for x in 0..width {
            let source_x = x * WIDTH / width;
            let source_offset = (source_y * WIDTH + source_x) * 4;
            let target_offset = (y * width + x) * 4;
            target[target_offset..target_offset + 4]
                .copy_from_slice(&source[source_offset..source_offset + 4]);
        }
    }
    target
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaled_frame_has_exact_dimensions_and_opaque_pixels() {
        let pixels = render_scaled(800, 600);
        assert_eq!(pixels.len(), 800 * 600 * 4);
        assert!(pixels.chunks_exact(4).all(|pixel| pixel[3] == 255));
    }

    #[test]
    fn scaled_frame_preserves_source_size() {
        let source = render_scaled(WIDTH, HEIGHT);
        assert_eq!(source.len(), WIDTH * HEIGHT * 4);
    }
}
