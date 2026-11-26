// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Coordinates compositor-side desktop appearance and surface policy.

use super::{DesktopBackground, DesktopSnapshot, DesktopSurface, DesktopTheme, WallpaperManager};

/// Owns desktop appearance policy consumed by each render frame.
///
/// Output discovery, window management, input and Vulkan resource ownership
/// remain outside this manager.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DesktopManager {
    theme: DesktopTheme,
    background: DesktopBackground,
    wallpaper: WallpaperManager,
}

impl DesktopManager {
    /// Returns the active desktop theme.
    pub const fn theme(&self) -> DesktopTheme {
        self.theme
    }

    /// Changes the theme without overriding an explicitly selected background.
    pub fn set_theme(&mut self, theme: DesktopTheme) {
        self.theme = theme;
    }

    /// Returns the configured background source.
    pub const fn background(&self) -> DesktopBackground {
        self.background
    }

    /// Selects the desktop background source.
    pub fn set_background(&mut self, background: DesktopBackground) {
        self.background = background;
    }

    /// Restores the background associated with the active theme.
    pub fn use_theme_background(&mut self) {
        self.background = DesktopBackground::ThemeDefault;
    }

    /// Returns the wallpaper selection manager.
    pub const fn wallpaper_manager(&self) -> &WallpaperManager {
        &self.wallpaper
    }

    /// Returns mutable access to the wallpaper selection manager.
    pub fn wallpaper_manager_mut(&mut self) -> &mut WallpaperManager {
        &mut self.wallpaper
    }

    /// Captures immutable desktop and wallpaper data for the next render frame.
    pub fn snapshot(&self) -> DesktopSnapshot {
        let color = self.background.resolve(self.theme);
        DesktopSnapshot::new(DesktopSurface::new(color), self.wallpaper.snapshot())
    }
}

#[cfg(test)]
mod tests {
    use super::{DesktopBackground, DesktopManager, DesktopTheme};

    #[test]
    fn default_manager_produces_light_desktop_surface_and_selected_wallpaper() {
        let manager = DesktopManager::default();
        assert_eq!(
            manager.snapshot().surface().background_color(),
            DesktopTheme::Light.desktop_color()
        );
        assert_eq!(
            manager
                .snapshot()
                .wallpaper()
                .map(|wallpaper| wallpaper.asset().id()),
            Some("aureaos-midnight-flow")
        );
    }

    #[test]
    fn theme_updates_the_default_background() {
        let mut manager = DesktopManager::default();
        manager.set_theme(DesktopTheme::Dark);
        assert_eq!(
            manager.snapshot().surface().background_color(),
            DesktopTheme::Dark.desktop_color()
        );
    }

    #[test]
    fn custom_background_is_independent_from_theme() {
        let mut manager = DesktopManager::default();
        manager.set_background(DesktopBackground::solid_color([0.1, 0.2, 0.3, 1.0]).unwrap());
        manager.set_theme(DesktopTheme::Dark);
        assert_eq!(
            manager.snapshot().surface().background_color(),
            [0.1, 0.2, 0.3, 1.0]
        );
        manager.use_theme_background();
        assert_eq!(
            manager.snapshot().surface().background_color(),
            DesktopTheme::Dark.desktop_color()
        );
    }
}
