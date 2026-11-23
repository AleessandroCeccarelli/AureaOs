// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Selects the desktop wallpaper and exposes immutable frame snapshots.

use super::{WallpaperAsset, WallpaperPlaybackPolicy, WallpaperSnapshot};

/// Owns wallpaper selection and playback policy independently from Vulkan
/// resource lifetimes. Decoder/provider instances are owned by worker services.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallpaperManager {
    selected: Option<WallpaperAsset>,
    playback_policy: WallpaperPlaybackPolicy,
}

impl WallpaperManager {
    /// Creates a manager using the wallpaper selected for the first desktop.
    pub fn with_default_wallpaper() -> Self {
        Self {
            selected: WallpaperAsset::default_wallpaper(),
            playback_policy: WallpaperPlaybackPolicy::default(),
        }
    }

    pub const fn selected(&self) -> Option<WallpaperAsset> {
        self.selected
    }

    pub const fn playback_policy(&self) -> WallpaperPlaybackPolicy {
        self.playback_policy
    }

    /// Updates animation behavior without changing the selected wallpaper.
    pub fn set_playback_policy(&mut self, policy: WallpaperPlaybackPolicy) {
        self.playback_policy = policy;
    }

    pub fn snapshot(&self) -> Option<WallpaperSnapshot> {
        self.selected.map(|asset| WallpaperSnapshot::new(asset, self.playback_policy))
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
            playback_policy: WallpaperPlaybackPolicy::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WallpaperManager;
    use crate::desktop::wallpaper::WallpaperPlaybackPolicy;

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

    #[test]
    fn playback_policy_is_carried_into_the_frame_snapshot() {
        let mut manager = WallpaperManager::default();
        let policy = WallpaperPlaybackPolicy::default().with_reduced_motion(true);
        manager.set_playback_policy(policy);

        assert!(manager.snapshot().unwrap().playback_policy().reduced_motion());
    }
}
