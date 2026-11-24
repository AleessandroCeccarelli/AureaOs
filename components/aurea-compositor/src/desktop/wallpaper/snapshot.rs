// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Immutable wallpaper data retained by one desktop render snapshot.

use super::{WallpaperAsset, WallpaperPlaybackPolicy};

/// Owned wallpaper source data and its playback policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WallpaperSnapshot {
    asset: WallpaperAsset,
    playback_policy: WallpaperPlaybackPolicy,
}

impl WallpaperSnapshot {
    pub(super) fn new(
        asset: WallpaperAsset,
        playback_policy: WallpaperPlaybackPolicy,
    ) -> Self {
        Self { asset, playback_policy }
    }

    pub fn asset(&self) -> &WallpaperAsset { &self.asset }
    pub const fn playback_policy(&self) -> WallpaperPlaybackPolicy { self.playback_policy }
}
