// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Immutable still-image wallpaper data retained by one desktop render snapshot.

use super::WallpaperAsset;

/// Owned still-image wallpaper source data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WallpaperSnapshot {
    asset: WallpaperAsset,
}

impl WallpaperSnapshot {
    pub(super) fn new(asset: WallpaperAsset) -> Self {
        Self { asset }
    }

    pub fn asset(&self) -> &WallpaperAsset { &self.asset }
}
