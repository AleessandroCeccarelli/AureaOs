// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Compositor-owned desktop surface policy.

mod background;
#[allow(dead_code)]
// Loader is staged until the runtime asset root is supplied by the composition root.
mod icon_asset_loader;
#[allow(dead_code)]
// Asset descriptors are staged until Fedora-generated RGBA8 assets are available.
mod icon_assets;
#[allow(dead_code)] // Geometry is staged before the dedicated desktop texture renderer.
mod item_geometry;
mod items_snapshot;
mod manager;
mod snapshot;
mod surface;
mod theme;
mod wallpaper;

pub use background::DesktopBackground;
#[allow(unused_imports)]
pub(crate) use icon_asset_loader::{DesktopIconAssetLoadError, load_desktop_icon_snapshot};
pub(crate) use icon_assets::{
    DesktopIconAssetDescriptor, DesktopIconTextureKey, desktop_context_menu_asset,
};
#[cfg(feature = "legacy-vulkan-frame")]
pub(crate) use icon_assets::desktop_icon_asset;
#[allow(unused_imports)]
pub(crate) use item_geometry::{
    DESKTOP_ICON_LOGICAL_SIZE, DesktopItemDrawGeometry, desktop_item_geometries,
};
pub use items_snapshot::DesktopItemsSnapshot;
pub use manager::DesktopManager;
pub use snapshot::DesktopSnapshot;
pub use surface::DesktopSurface;
pub use theme::DesktopTheme;
pub use wallpaper::{
    DecodedWallpaper, WallpaperAsset, WallpaperAssetError, WallpaperDecodeKey,
    WallpaperDecodeService, WallpaperDecodeServiceError, WallpaperDimensions,
    WallpaperDrawGeometry, WallpaperFitMode, WallpaperFormat, WallpaperLayout, WallpaperManager,
    WallpaperPixelRect, WallpaperPixels, WallpaperPixelsError, WallpaperSnapshot, WallpaperUvRect,
};
