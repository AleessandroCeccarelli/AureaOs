// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Compositor-owned desktop surface policy.

mod background;
mod items_snapshot;
#[allow(dead_code)] // Asset descriptors are staged until Fedora-generated RGBA8 assets are available.
mod icon_assets;
#[allow(dead_code)] // Geometry is staged before the dedicated desktop texture renderer.
mod item_geometry;
mod manager;
mod snapshot;
mod surface;
mod theme;
mod wallpaper;

pub use background::DesktopBackground;
pub use items_snapshot::DesktopItemsSnapshot;
#[allow(unused_imports)]
pub(crate) use icon_assets::{
    DesktopIconAssetDescriptor, DesktopIconRasterSize, DesktopIconTextureKey,
    desktop_icon_asset,
};
#[allow(unused_imports)]
pub(crate) use item_geometry::{
    DESKTOP_ICON_LOGICAL_SIZE, DesktopItemDrawGeometry, desktop_item_geometries,
};
pub use manager::DesktopManager;
pub use snapshot::DesktopSnapshot;
pub use surface::DesktopSurface;
pub use theme::DesktopTheme;
pub use wallpaper::{
    DecodedWallpaper, WallpaperAsset, WallpaperDecodeKey, WallpaperDecodeService,
    WallpaperDecodeServiceError, WallpaperDimensions, WallpaperDrawGeometry, WallpaperFitMode,
    WallpaperFormat, WallpaperLayout, WallpaperManager, WallpaperPixelRect, WallpaperPixels,
    WallpaperPixelsError, WallpaperSnapshot, WallpaperUvRect,
};
