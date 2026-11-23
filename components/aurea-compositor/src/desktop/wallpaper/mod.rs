// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Desktop wallpaper asset metadata and frame snapshots.

mod asset;
mod manager;
mod pixels;
mod layout;
mod snapshot;

pub use asset::{WallpaperAsset, WallpaperDimensions, WallpaperFormat};
pub use manager::WallpaperManager;
pub use layout::{WallpaperFitMode, WallpaperLayout, WallpaperPixelRect};
pub use snapshot::WallpaperSnapshot;

pub use pixels::{WallpaperPixels, WallpaperPixelsError, MAX_WALLPAPER_RGBA_BYTES};
