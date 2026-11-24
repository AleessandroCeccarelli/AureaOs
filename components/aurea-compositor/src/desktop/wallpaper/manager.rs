// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Selects a still-image desktop wallpaper and exposes immutable snapshots.

use super::{WallpaperAsset, WallpaperSnapshot};

/// Owns still-image wallpaper selection independently from Vulkan resources.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WallpaperManager {
    selected: Option<WallpaperAsset>,
}

impl WallpaperManager {
    pub fn with_default_wallpaper() -> Self {
        Self { selected: WallpaperAsset::default_wallpaper() }
    }

    pub fn selected(&self) -> Option<&WallpaperAsset> { self.selected.as_ref() }

    pub fn snapshot(&self) -> Option<WallpaperSnapshot> {
        self.selected.clone().map(WallpaperSnapshot::new)
    }

    pub fn select(&mut self, asset: WallpaperAsset) { self.selected = Some(asset); }
    pub fn clear_selection(&mut self) { self.selected = None; }
}

impl Default for WallpaperManager {
    fn default() -> Self { Self::with_default_wallpaper() }
}

impl From<WallpaperAsset> for WallpaperManager {
    fn from(asset: WallpaperAsset) -> Self { Self { selected: Some(asset) } }
}

#[cfg(test)]
mod tests {
    use super::WallpaperManager;
    use crate::desktop::wallpaper::WallpaperAsset;

    #[test]
    fn default_manager_selects_the_aureaos_wallpaper() {
        let manager = WallpaperManager::default();
        assert_eq!(manager.selected().map(|asset| asset.id()), Some("aureaos-midnight-flow"));
    }

    #[test]
    fn manager_accepts_a_user_owned_image_asset() {
        let asset = WallpaperAsset::from_bytes("user-test", b"BM".to_vec()).unwrap();
        let mut manager = WallpaperManager::default();
        manager.select(asset);
        assert_eq!(manager.selected().map(|asset| asset.id()), Some("user-test"));
    }

    #[test]
    fn clearing_selection_enables_the_background_fallback() {
        let mut manager = WallpaperManager::default();
        manager.clear_selection();
        assert!(manager.snapshot().is_none());
    }
}
