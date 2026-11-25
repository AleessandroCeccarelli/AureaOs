// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Output-constrained placement for transient desktop panels.

use super::{DesktopPoint, DesktopRect};

/// The final logical bounds of a transient panel on one output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopPanelPlacement {
    bounds: DesktopRect,
}

impl DesktopPanelPlacement {
    /// Places a panel at its pointer anchor and keeps the complete panel inside
    /// the output. The caller supplies content-driven dimensions in logical
    /// units; oversized panels must be laid out or scrolled before placement.
    pub fn context_menu(
        anchor: DesktopPoint,
        output: DesktopRect,
        panel_width: u32,
        panel_height: u32,
    ) -> Option<Self> {
        if panel_width == 0
            || panel_height == 0
            || panel_width > output.width()
            || panel_height > output.height()
        {
            return None;
        }

        let output_origin = output.origin();
        let left = i64::from(output_origin.x);
        let top = i64::from(output_origin.y);
        let right = left + i64::from(output.width());
        let bottom = top + i64::from(output.height());
        let max_x = right - i64::from(panel_width);
        let max_y = bottom - i64::from(panel_height);
        let x = i64::from(anchor.x).clamp(left, max_x);
        let y = i64::from(anchor.y).clamp(top, max_y);

        let origin = DesktopPoint::new(i32::try_from(x).ok()?, i32::try_from(y).ok()?);
        Some(Self {
            bounds: DesktopRect::new(origin, panel_width, panel_height)?,
        })
    }

    pub const fn bounds(self) -> DesktopRect {
        self.bounds
    }
}

#[cfg(test)]
mod tests {
    use super::DesktopPanelPlacement;
    use crate::desktop::{DesktopPoint, DesktopRect};

    fn output() -> DesktopRect {
        DesktopRect::new(DesktopPoint::new(100, 50), 1_000, 700).unwrap()
    }

    #[test]
    fn keeps_menu_at_pointer_when_there_is_room() {
        let placement = DesktopPanelPlacement::context_menu(
            DesktopPoint::new(300, 200),
            output(),
            240,
            320,
        )
        .unwrap();

        assert_eq!(placement.bounds().origin(), DesktopPoint::new(300, 200));
    }

    #[test]
    fn moves_menu_left_and_up_near_bottom_right_edge() {
        let placement = DesktopPanelPlacement::context_menu(
            DesktopPoint::new(1_050, 720),
            output(),
            240,
            320,
        )
        .unwrap();

        assert_eq!(placement.bounds().origin(), DesktopPoint::new(860, 430));
        assert!(output().contains(placement.bounds().origin()));
    }

    #[test]
    fn clamps_anchor_to_output_when_pointer_is_outside_it() {
        let placement = DesktopPanelPlacement::context_menu(
            DesktopPoint::new(-500, -500),
            output(),
            240,
            320,
        )
        .unwrap();

        assert_eq!(placement.bounds().origin(), DesktopPoint::new(100, 50));
    }

    #[test]
    fn rejects_panels_larger_than_the_output_or_with_zero_size() {
        assert!(DesktopPanelPlacement::context_menu(
            DesktopPoint::new(100, 50),
            output(),
            1_001,
            20,
        )
        .is_none());
        assert!(DesktopPanelPlacement::context_menu(
            DesktopPoint::new(100, 50),
            output(),
            20,
            0,
        )
        .is_none());
    }
}
