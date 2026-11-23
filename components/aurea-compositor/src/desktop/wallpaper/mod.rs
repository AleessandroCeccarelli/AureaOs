// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Desktop wallpaper asset metadata and frame snapshots.

mod asset;
mod manager;
mod snapshot;

pub use asset::{WallpaperAsset, WallpaperDimensions, WallpaperFormat};
pub use manager::WallpaperManager;
pub use snapshot::WallpaperSnapshot;
