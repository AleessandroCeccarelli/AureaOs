// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Technology-independent desktop context-menu commands and hit geometry.

use super::{DesktopPanelPlacement, DesktopPoint, DesktopRect};

/// A command requested by selecting an item in the desktop context menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DesktopContextMenuCommand {
    CreateFolder,
    UseAsWallpaper,
    OpenDisplaySettings,
    OpenPersonalization,
}

/// One actionable row in the desktop context menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopContextMenuItem {
    command: DesktopContextMenuCommand,
    bounds: DesktopRect,
}

impl DesktopContextMenuItem {
    pub const fn command(self) -> DesktopContextMenuCommand {
        self.command
    }

    pub const fn bounds(self) -> DesktopRect {
        self.bounds
    }
}

/// Immutable geometry for the standard desktop context menu.
///
/// The menu is deliberately described in logical coordinates. Each output
/// computes its own placement, so mixed-scale multi-monitor layouts do not
/// share physical-pixel assumptions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopContextMenuLayout {
    placement: DesktopPanelPlacement,
    items: [DesktopContextMenuItem; 4],
}

impl DesktopContextMenuLayout {
    pub const WIDTH: u32 = 248;
    pub const ROW_HEIGHT: u32 = 36;
    pub const VERTICAL_PADDING: u32 = 8;
    pub const HEIGHT: u32 = Self::ROW_HEIGHT * 4 + Self::VERTICAL_PADDING * 2;

    /// Creates the default desktop menu at the pointer, constrained to output.
    pub fn at(
        anchor: DesktopPoint,
        output: DesktopRect,
    ) -> Option<Self> {
        let placement = DesktopPanelPlacement::context_menu(
            anchor,
            output,
            Self::WIDTH,
            Self::HEIGHT,
        )?;
        let origin = placement.bounds().origin();
        let first_row_y = origin.y.checked_add(Self::VERTICAL_PADDING as i32)?;
        let commands = [
            DesktopContextMenuCommand::CreateFolder,
            DesktopContextMenuCommand::UseAsWallpaper,
            DesktopContextMenuCommand::OpenDisplaySettings,
            DesktopContextMenuCommand::OpenPersonalization,
        ];
        let items = std::array::from_fn(|index| {
            let row_origin = DesktopPoint::new(
                origin.x,
                first_row_y + index as i32 * Self::ROW_HEIGHT as i32,
            );
            DesktopContextMenuItem {
                command: commands[index],
                bounds: DesktopRect::new(row_origin, Self::WIDTH, Self::ROW_HEIGHT)
                    .expect("validated non-zero context-menu row"),
            }
        });
        Some(Self { placement, items })
    }

    pub const fn placement(&self) -> DesktopPanelPlacement {
        self.placement
    }

    pub fn items(&self) -> &[DesktopContextMenuItem; 4] {
        &self.items
    }

    /// Returns the command under a global logical pointer position.
    pub fn command_at(&self, point: DesktopPoint) -> Option<DesktopContextMenuCommand> {
        self.items
            .iter()
            .find(|item| item.bounds.contains(point))
            .map(|item| item.command)
    }

    /// Returns true for the menu background, including padding, but not rows.
    pub fn contains(&self, point: DesktopPoint) -> bool {
        self.placement.bounds().contains(point)
    }
}

#[cfg(test)]
mod tests {
    use super::{DesktopContextMenuCommand, DesktopContextMenuLayout};
    use crate::desktop::{DesktopPoint, DesktopRect};

    fn output() -> DesktopRect {
        DesktopRect::new(DesktopPoint::new(100, 50), 1_000, 700).unwrap()
    }

    #[test]
    fn lays_out_four_rows_with_stable_commands() {
        let menu = DesktopContextMenuLayout::at(DesktopPoint::new(300, 200), output()).unwrap();
        assert_eq!(menu.items().len(), 4);
        assert_eq!(
            menu.command_at(DesktopPoint::new(310, 220)),
            Some(DesktopContextMenuCommand::CreateFolder)
        );
        assert_eq!(
            menu.command_at(DesktopPoint::new(310, 256)),
            Some(DesktopContextMenuCommand::ChangeWallpaper)
        );
        assert_eq!(
            menu.command_at(DesktopPoint::new(310, 292)),
            Some(DesktopContextMenuCommand::OpenDisplaySettings)
        );
        assert_eq!(
            menu.command_at(DesktopPoint::new(310, 328)),
            Some(DesktopContextMenuCommand::OpenPersonalization)
        );
    }

    #[test]
    fn constrains_menu_and_hit_regions_to_the_selected_output() {
        let menu = DesktopContextMenuLayout::at(DesktopPoint::new(1_050, 720), output()).unwrap();
        let bounds = menu.placement().bounds();
        assert_eq!(bounds.origin(), DesktopPoint::new(852, 590));
        assert!(output().contains(bounds.origin()));
        assert!(menu.contains(bounds.origin()));
    }

    #[test]
    fn distinguishes_menu_padding_from_action_rows() {
        let menu = DesktopContextMenuLayout::at(DesktopPoint::new(300, 200), output()).unwrap();
        let origin = menu.placement().bounds().origin();
        assert!(menu.contains(origin));
        assert_eq!(
            menu.command_at(origin),
            None,
            "top padding must not activate a menu command"
        );
    }

    #[test]
    fn rejects_outputs_that_cannot_fit_the_menu() {
        let small = DesktopRect::new(DesktopPoint::new(0, 0), 200, 500).unwrap();
        assert!(DesktopContextMenuLayout::at(DesktopPoint::new(10, 10), small).is_none());
    }
}
