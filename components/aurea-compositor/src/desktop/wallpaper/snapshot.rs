// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Immutable wallpaper data retained by one desktop render snapshot.

use super::WallpaperAsset;

/// Borrowed, frame-safe wallpaper source data.
#[derive(Debug, Clone, Copy)]
pub struct WallpaperSnapshot {
    asset: WallpaperAsset,
}

impl WallpaperSnapshot {
    pub(super) const fn new(asset: WallpaperAsset) -> Self {
        Self { asset }
    }

    pub const fn asset(self) -> WallpaperAsset {
        self.asset
    }
}
