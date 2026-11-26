// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Desktop still-image wallpaper assets, layout, and decoded pixels.

mod asset;
mod decode_service;
mod draw_geometry;
mod layout;
mod manager;
mod pixels;
mod snapshot;

pub use asset::{WallpaperAsset, WallpaperAssetError, WallpaperDimensions, WallpaperFormat};
pub use decode_service::{
    DecodedWallpaper, WallpaperDecodeKey, WallpaperDecodeService, WallpaperDecodeServiceError,
};
pub use draw_geometry::{WallpaperDrawGeometry, WallpaperUvRect};
pub use layout::{WallpaperFitMode, WallpaperLayout, WallpaperPixelRect};
pub use manager::WallpaperManager;
pub use pixels::{MAX_WALLPAPER_RGBA_BYTES, WallpaperPixels, WallpaperPixelsError};
pub use snapshot::WallpaperSnapshot;
