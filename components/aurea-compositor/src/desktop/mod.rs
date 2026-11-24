// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Compositor-owned desktop surface policy.

mod background;
mod items_snapshot;
mod manager;
mod snapshot;
mod surface;
mod theme;
mod wallpaper;

pub use background::DesktopBackground;
pub use items_snapshot::DesktopItemsSnapshot;
pub use manager::DesktopManager;
pub use snapshot::DesktopSnapshot;
pub use surface::DesktopSurface;
pub use theme::DesktopTheme;
pub use wallpaper::{
    WallpaperAsset, WallpaperDimensions, WallpaperFormat, WallpaperManager, WallpaperPixels,
    WallpaperPixelsError, WallpaperSnapshot,
};
