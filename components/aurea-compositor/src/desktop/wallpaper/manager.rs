// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Selects the desktop wallpaper and exposes immutable frame snapshots.

use super::{WallpaperAsset, WallpaperSnapshot};

/// Owns wallpaper selection independently from Vulkan resource lifetimes.
///
/// The initial asset is embedded in the compositor. JPEG pixel decoding and
/// GPU texture creation are separate pipeline stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallpaperManager {
    selected: Option<WallpaperAsset>,
}

impl WallpaperManager {
    /// Creates a manager using the wallpaper selected for the first desktop.
    pub fn with_default_wallpaper() -> Self {
        Self {
            selected: WallpaperAsset::default_wallpaper(),
        }
    }

    pub const fn selected(&self) -> Option<WallpaperAsset> {
        self.selected
    }

    pub fn snapshot(&self) -> Option<WallpaperSnapshot> {
        self.selected.map(WallpaperSnapshot::new)
    }

    pub fn select(&mut self, asset: WallpaperAsset) {
        self.selected = Some(asset);
    }

    /// Disables the wallpaper so the desktop theme color is used as fallback.
    pub fn clear_selection(&mut self) {
        self.selected = None;
    }
}

impl Default for WallpaperManager {
    fn default() -> Self {
        Self::with_default_wallpaper()
    }
}

impl From<WallpaperAsset> for WallpaperManager {
    fn from(asset: WallpaperAsset) -> Self {
        Self {
            selected: Some(asset),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WallpaperManager;

    #[test]
    fn default_manager_selects_the_aureaos_wallpaper() {
        let manager = WallpaperManager::default();
        assert_eq!(
            manager.selected().map(|asset| asset.id()),
            Some("aureaos-midnight-flow")
        );
    }

    #[test]
    fn clearing_selection_enables_the_background_fallback() {
        let mut manager = WallpaperManager::default();
        manager.clear_selection();
        assert!(manager.snapshot().is_none());
    }
}
