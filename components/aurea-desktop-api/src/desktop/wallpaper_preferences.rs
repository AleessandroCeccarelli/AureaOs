// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Technology-independent wallpaper preferences shared by desktop components.

/// How a still-image wallpaper is placed inside each display output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WallpaperScaleMode {
    /// Preserve aspect ratio and crop excess image area.
    #[default]
    Fill,
    /// Preserve aspect ratio and show the complete image.
    Fit,
    /// Resize independently on both axes.
    Stretch,
    /// Keep native pixel dimensions and center the image.
    Center,
}

/// Persistent user choice for the desktop wallpaper.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WallpaperPreferences {
    image_path: Option<String>,
    scale_mode: WallpaperScaleMode,
}

impl WallpaperPreferences {
    pub fn image_path(&self) -> Option<&str> {
        self.image_path.as_deref()
    }

    /// Sets a path to a user-owned still image. Empty paths clear the selection.
    pub fn set_image_path(&mut self, path: Option<String>) {
        self.image_path = path.filter(|value| !value.trim().is_empty());
    }

    pub const fn scale_mode(&self) -> WallpaperScaleMode {
        self.scale_mode
    }

    pub fn set_scale_mode(&mut self, mode: WallpaperScaleMode) {
        self.scale_mode = mode;
    }
}

#[cfg(test)]
mod tests {
    use super::{WallpaperPreferences, WallpaperScaleMode};

    #[test]
    fn defaults_to_theme_wallpaper_with_fill_mode() {
        let preferences = WallpaperPreferences::default();
        assert_eq!(preferences.image_path(), None);
        assert_eq!(preferences.scale_mode(), WallpaperScaleMode::Fill);
    }

    #[test]
    fn empty_path_clears_user_selection() {
        let mut preferences = WallpaperPreferences::default();
        preferences.set_image_path(Some("/home/user/Pictures/wall.jpg".into()));
        preferences.set_image_path(Some("  ".into()));
        assert_eq!(preferences.image_path(), None);
    }
}
