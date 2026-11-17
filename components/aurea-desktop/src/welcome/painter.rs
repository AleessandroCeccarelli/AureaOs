// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Pixel-level composition of the first AureaOS Welcome surface.

use super::{HEIGHT, WIDTH, typography::draw_text};

const NAVY: [u8; 4] = [10, 20, 36, 255];
const GOLD: [u8; 4] = [232, 184, 82, 255];
const WHITE: [u8; 4] = [239, 244, 250, 255];
const MUTED: [u8; 4] = [137, 160, 190, 255];

pub(super) fn paint_welcome(pixels: &mut [u8]) {
    paint_background(pixels);
    paint_glow(pixels, 128, 210, 150, [35, 92, 150, 68]);
    paint_glow(pixels, 130, 208, 82, [208, 145, 45, 35]);

    fill_round_rect(pixels, 24, 18, 672, 50, 15, [17, 34, 57, 245]);
    draw_text(pixels, 52, 35, "AUREAOS", 2, WHITE);
    draw_text(pixels, 574, 37, "PREVIEW", 2, GOLD);

    fill_round_rect(pixels, 24, 92, 672, 310, 22, [16, 31, 52, 250]);
    stroke_round_rect(pixels, 24, 92, 672, 310, 22, [62, 91, 126, 180]);

    // A softly illuminated tile containing the geometric Aurea "A".
    fill_round_rect(pixels, 72, 148, 112, 112, 24, [239, 194, 103, 255]);
    fill_round_rect(pixels, 80, 156, 96, 96, 19, [224, 169, 68, 255]);
    draw_line(pixels, (128, 174), (101, 231), 7, NAVY);
    draw_line(pixels, (128, 174), (155, 231), 7, NAVY);
    draw_line(pixels, (112, 211), (144, 211), 6, NAVY);
    fill_round_rect(pixels, 122, 232, 12, 5, 2, [255, 231, 174, 255]);

    draw_text(pixels, 224, 153, "BENVENUTO", 3, WHITE);
    draw_text(pixels, 224, 190, "IN AUREAOS", 3, GOLD);
    draw_text(pixels, 224, 246, "IL DESKTOP PRENDE FORMA", 2, MUTED);

    fill_round_rect(pixels, 72, 300, 576, 2, 1, [61, 87, 119, 255]);
    draw_text(
        pixels,
        72,
        326,
        "WAYLAND  VULKAN  SMITHAY",
        2,
        [183, 202, 224, 255],
    );

    // A quiet progress motif: a bright first segment, not a fake loading bar.
    fill_round_rect(pixels, 72, 366, 576, 4, 2, [37, 55, 78, 255]);
    fill_round_rect(pixels, 72, 366, 164, 4, 2, GOLD);
}

fn paint_background(pixels: &mut [u8]) {
    for y in 0..HEIGHT {
        let progress = y as u32 * 255 / HEIGHT as u32;
        let red = 7 + progress * 7 / 255;
        let green = 13 + progress * 13 / 255;
        let blue = 25 + progress * 25 / 255;
        for x in 0..WIDTH {
            let offset = (y * WIDTH + x) * 4;
            pixels[offset..offset + 4].copy_from_slice(&[blue as u8, green as u8, red as u8, 255]);
        }
    }
}

fn paint_glow(pixels: &mut [u8], center_x: usize, center_y: usize, radius: usize, color: [u8; 4]) {
    let radius_squared = radius * radius;
    let start_x = center_x.saturating_sub(radius);
    let end_x = (center_x + radius).min(WIDTH - 1);
    let start_y = center_y.saturating_sub(radius);
    let end_y = (center_y + radius).min(HEIGHT - 1);

    for y in start_y..=end_y {
        for x in start_x..=end_x {
            let dx = x.abs_diff(center_x);
            let dy = y.abs_diff(center_y);
            let distance_squared = dx * dx + dy * dy;
            if distance_squared >= radius_squared {
                continue;
            }
            let falloff = (radius_squared - distance_squared) * color[3] as usize / radius_squared;
            blend_pixel(pixels, x, y, [color[0], color[1], color[2], falloff as u8]);
        }
    }
}

fn fill_round_rect(
    pixels: &mut [u8],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    radius: usize,
    color: [u8; 4],
) {
    if width == 0 || height == 0 {
        return;
    }
    let radius = radius.min(width / 2).min(height / 2);
    for row in y..(y + height).min(HEIGHT) {
        for column in x..(x + width).min(WIDTH) {
            let nearest_x = column.clamp(x + radius, x + width - radius - 1);
            let nearest_y = row.clamp(y + radius, y + height - radius - 1);
            let dx = column.abs_diff(nearest_x);
            let dy = row.abs_diff(nearest_y);
            if dx * dx + dy * dy <= radius * radius {
                blend_pixel(pixels, column, row, color);
            }
        }
    }
}

fn stroke_round_rect(
    pixels: &mut [u8],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    radius: usize,
    color: [u8; 4],
) {
    fill_round_rect(pixels, x, y, width, 1, radius, color);
    fill_round_rect(pixels, x, y + height - 1, width, 1, radius, color);
    fill_round_rect(pixels, x, y, 1, height, radius, color);
    fill_round_rect(pixels, x + width - 1, y, 1, height, radius, color);
}

fn draw_line(
    pixels: &mut [u8],
    start: (usize, usize),
    end: (usize, usize),
    thickness: usize,
    color: [u8; 4],
) {
    let (mut x0, mut y0) = (start.0 as isize, start.1 as isize);
    let (x1, y1) = (end.0 as isize, end.1 as isize);
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;
    let radius = thickness / 2;

    loop {
        fill_round_rect(
            pixels,
            (x0.max(0) as usize).saturating_sub(radius),
            (y0.max(0) as usize).saturating_sub(radius),
            thickness,
            thickness,
            radius,
            color,
        );
        if x0 == x1 && y0 == y1 {
            break;
        }
        let doubled = 2 * error;
        if doubled >= dy {
            error += dy;
            x0 += sx;
        }
        if doubled <= dx {
            error += dx;
            y0 += sy;
        }
    }
}

fn blend_pixel(pixels: &mut [u8], x: usize, y: usize, color: [u8; 4]) {
    if x >= WIDTH || y >= HEIGHT {
        return;
    }
    let offset = (y * WIDTH + x) * 4;
    let alpha = u16::from(color[3]);
    let inverse = 255 - alpha;
    for channel in 0..3 {
        let source = u16::from(color[2 - channel]);
        let destination = u16::from(pixels[offset + channel]);
        pixels[offset + channel] = ((source * alpha + destination * inverse) / 255) as u8;
    }
    pixels[offset + 3] = 255;
}

#[cfg(test)]
mod tests {
    use super::{HEIGHT, NAVY, WIDTH, paint_welcome};

    #[test]
    fn welcome_surface_has_a_dark_blue_gradient() {
        let mut pixels = vec![0; WIDTH * HEIGHT * 4];
        paint_welcome(&mut pixels);

        let pixel = |x: usize, y: usize| {
            let offset = (y * WIDTH + x) * 4;
            &pixels[offset..offset + 4]
        };

        assert_eq!(pixel(0, 0), [25, 13, 7, 255]);
        assert_ne!(pixel(0, 0), pixel(0, HEIGHT - 1));
    }

    #[test]
    fn welcome_surface_contains_the_gold_mark_and_legible_title() {
        let mut pixels = vec![0; WIDTH * HEIGHT * 4];
        paint_welcome(&mut pixels);

        let pixel = |x: usize, y: usize| {
            let offset = (y * WIDTH + x) * 4;
            &pixels[offset..offset + 4]
        };

        assert_eq!(pixel(96, 160), [224, 169, 68, 255]);
        assert_eq!(pixel(128, 211), [NAVY[2], NAVY[1], NAVY[0], 255]);
        assert_eq!(pixel(224, 153), [250, 244, 239, 255]);
        assert_ne!(pixel(224, 153), pixel(236, 153));
    }
}
