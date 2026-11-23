// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Selects the desktop wallpaper and exposes immutable frame snapshots.

use super::{WallpaperAsset, WallpaperSnapshot};

/// Owns wallpaper selection independently from Vulkan resource lifetimes.
///
/// The initial asset is embedded in the compositor. JPEG pixel decoding and
/// GPU texture creation are deliberately delegated to later pipeline stages.
#[derive(Debug, Clone, Copy, Default)]
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

    /// Returns the currently selected validated wallpaper, if available.
    pub const fn selected(&self) -> Option<WallpaperAsset> {
        self.selected
    }

    /// Captures the selected wallpaper for an immutable render frame.
    pub fn snapshot(&self) -> Option<WallpaperSnapshot> {
        self.selected.map(WallpaperSnapshot::new)
    }

    /// Replaces the current selection with a validated embedded asset.
    pub fn select(&mut self, asset: WallpaperAsset) {
        self.selected = Some(asset);
    }

    /// Disables the wallpaper so the desktop theme color is used as fallback.
    pub fn clear_selection(&mut self) {
        self.selected = None;
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
        let manager = WallpaperManager::with_default_wallpaper();
        assert_eq!(
            manager.selected().map(|asset| asset.id()),
            Some("aureaos-midnight-flow")
        );
    }

    #[test]
    fn clearing_selection_enables_the_background_fallback() {
        let mut manager = WallpaperManager::with_default_wallpaper();
        manager.clear_selection();
        assert!(manager.snapshot().is_none());
    }
}
