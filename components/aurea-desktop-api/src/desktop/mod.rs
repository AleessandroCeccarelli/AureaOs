// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Technology-independent contracts for the user's desktop space.

mod context_menu;
mod item;
mod layout;
mod manager;
mod panel_geometry;
mod preferences;
mod stacks;
mod trash_state;
mod wallpaper_preferences;

pub use context_menu::{
    DesktopContextMenuCommand, DesktopContextMenuItem, DesktopContextMenuLayout,
};
pub use item::{DesktopItem, DesktopItemId, DesktopItemKind, DesktopPosition};
pub use layout::{DesktopHitTest, DesktopPoint, DesktopRect};
pub use panel_geometry::DesktopPanelPlacement;
pub use manager::{DesktopManager, DesktopManagerError};
pub use preferences::{DesktopPreferences, StackGrouping};
pub use stacks::{DesktopStack, DesktopStackCategory, DesktopStackId};
pub use trash_state::DesktopTrashState;
pub use wallpaper_preferences::{WallpaperPreferences, WallpaperScaleMode};
