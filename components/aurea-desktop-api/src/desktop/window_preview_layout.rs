// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Output-local logical geometry for Overview and App Exposé window previews.

use super::{DesktopPoint, DesktopRect};

/// Stable window identity and committed logical dimensions used by a preview layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopWindowPreviewInput {
    pub id: String,
    pub width: u32,
    pub height: u32,
}

/// A window preview's destination rectangle and source identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopWindowPreview {
    id: String,
    bounds: DesktopRect,
}

impl DesktopWindowPreview {
    pub fn id(&self) -> &str { &self.id }
    pub const fn bounds(&self) -> DesktopRect { self.bounds }
}

/// Deterministic grid layout shared by Overview and App Exposé.
///
/// The output rectangle is in global logical coordinates. Preview rectangles
/// preserve each source window's aspect ratio and never extend beyond it.
pub struct DesktopWindowPreviewLayout;

impl DesktopWindowPreviewLayout {
    pub fn grid(
        output: DesktopRect,
        windows: &[DesktopWindowPreviewInput],
        columns: u32,
        margin: u32,
        gap: u32,
    ) -> Vec<DesktopWindowPreview> {
        if windows.is_empty() || columns == 0 { return Vec::new(); }

        let count = windows.len() as u32;
        let columns = columns.min(count);
        let rows = count.div_ceil(columns);
        let available_width = output.width().saturating_sub(margin.saturating_mul(2));
        let available_height = output.height().saturating_sub(margin.saturating_mul(2));
        if available_width == 0 || available_height == 0 { return Vec::new(); }

        let total_gap_x = gap.saturating_mul(columns.saturating_sub(1));
        let total_gap_y = gap.saturating_mul(rows.saturating_sub(1));
        let cell_width = available_width.saturating_sub(total_gap_x) / columns;
        let cell_height = available_height.saturating_sub(total_gap_y) / rows;
        if cell_width == 0 || cell_height == 0 { return Vec::new(); }

        let origin = output.origin();
        windows.iter().enumerate().filter_map(|(index, window)| {
            if window.id.is_empty() || window.width == 0 || window.height == 0 {
                return None;
            }
            let column = index as u32 % columns;
            let row = index as u32 / columns;
            let scale = (cell_width as f64 / window.width as f64)
                .min(cell_height as f64 / window.height as f64);
            if !scale.is_finite() || scale <= 0.0 { return None; }
            let width = ((window.width as f64 * scale).floor() as u32).max(1);
            let height = ((window.height as f64 * scale).floor() as u32).max(1);
            let cell_x = margin.saturating_add(column.saturating_mul(cell_width.saturating_add(gap)));
            let cell_y = margin.saturating_add(row.saturating_mul(cell_height.saturating_add(gap)));
            let x = cell_x.saturating_add(cell_width.saturating_sub(width) / 2);
            let y = cell_y.saturating_add(cell_height.saturating_sub(height) / 2);
            let x = i64::from(origin.x) + i64::from(x);
            let y = i64::from(origin.y) + i64::from(y);
            let point = DesktopPoint::new(i32::try_from(x).ok()?, i32::try_from(y).ok()?);
            Some(DesktopWindowPreview {
                id: window.id.clone(),
                bounds: DesktopRect::new(point, width, height)?,
            })
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::{DesktopPoint, DesktopRect};

    fn output() -> DesktopRect {
        DesktopRect::new(DesktopPoint::new(100, 50), 1_000, 700).unwrap()
    }

    #[test]
    fn grid_preserves_window_aspect_ratio_and_stays_inside_output() {
        let windows = [
            DesktopWindowPreviewInput { id: "wide".into(), width: 1600, height: 900 },
            DesktopWindowPreviewInput { id: "portrait".into(), width: 800, height: 1200 },
        ];
        let previews = DesktopWindowPreviewLayout::grid(output(), &windows, 2, 24, 16);
        assert_eq!(previews.len(), 2);
        for preview in previews {
            let bounds = preview.bounds();
            assert!(output().contains(bounds.origin()));
            assert!(bounds.origin().x as i64 + bounds.width() as i64 <= 1_100);
            assert!(bounds.origin().y as i64 + bounds.height() as i64 <= 750);
        }
        let wide = DesktopWindowPreviewLayout::grid(output(), &windows[..1], 1, 24, 16);
        let bounds = wide[0].bounds();
        assert!((bounds.width() as f64 / bounds.height() as f64 - 16.0 / 9.0).abs() < 0.01);
    }

    #[test]
    fn grid_uses_global_logical_output_origin() {
        let windows = [DesktopWindowPreviewInput { id: "one".into(), width: 800, height: 600 }];
        let previews = DesktopWindowPreviewLayout::grid(output(), &windows, 1, 20, 12);
        assert!(previews[0].bounds().origin().x >= 100);
        assert!(previews[0].bounds().origin().y >= 50);
    }

    #[test]
    fn invalid_windows_are_skipped_and_empty_layouts_are_safe() {
        let windows = [
            DesktopWindowPreviewInput { id: String::new(), width: 800, height: 600 },
            DesktopWindowPreviewInput { id: "invalid".into(), width: 0, height: 600 },
            DesktopWindowPreviewInput { id: "valid".into(), width: 800, height: 600 },
        ];
        let previews = DesktopWindowPreviewLayout::grid(output(), &windows, 2, 20, 12);
        assert_eq!(previews.len(), 1);
        assert!(DesktopWindowPreviewLayout::grid(output(), &windows, 0, 20, 12).is_empty());
        assert!(DesktopWindowPreviewLayout::grid(output(), &[], 2, 20, 12).is_empty());
    }

    #[test]
    fn excessive_spacing_produces_no_out_of_bounds_rectangles() {
        let windows = [DesktopWindowPreviewInput { id: "one".into(), width: 800, height: 600 }];
        assert!(DesktopWindowPreviewLayout::grid(output(), &windows, 1, 600, 12).is_empty());
    }
}
