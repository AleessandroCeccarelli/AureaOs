// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Selects and resolves the desktop background source.

use super::DesktopTheme;

/// Background source currently supported by the desktop surface.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DesktopBackground {
    /// Use the color associated with the active desktop theme.
    ThemeDefault,
    /// Use an explicit normalized RGBA color.
    SolidColor([f32; 4]),
}

impl Default for DesktopBackground {
    fn default() -> Self {
        Self::ThemeDefault
    }
}

impl DesktopBackground {
    /// Creates a solid background only when all channels are finite and normalized.
    pub fn solid_color(color: [f32; 4]) -> Option<Self> {
        color
            .iter()
            .all(|channel| channel.is_finite() && (0.0..=1.0).contains(channel))
            .then_some(Self::SolidColor(color))
    }

    /// Resolves the selected source to an opaque desktop color.
    pub fn resolve(self, theme: DesktopTheme) -> [f32; 4] {
        match self {
            Self::ThemeDefault => theme.desktop_color(),
            Self::SolidColor(mut color) => {
                color[3] = 1.0;
                color
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DesktopBackground;
    use crate::desktop::DesktopTheme;

    #[test]
    fn theme_background_tracks_theme_changes() {
        let background = DesktopBackground::default();
        assert_eq!(
            background.resolve(DesktopTheme::Light),
            DesktopTheme::Light.desktop_color()
        );
        assert_eq!(
            background.resolve(DesktopTheme::Dark),
            DesktopTheme::Dark.desktop_color()
        );
    }

    #[test]
    fn solid_color_rejects_invalid_channels_and_is_opaque() {
        assert!(DesktopBackground::solid_color([f32::NAN, 0.0, 0.0, 1.0]).is_none());
        assert!(DesktopBackground::solid_color([0.0, 1.1, 0.0, 1.0]).is_none());
        assert_eq!(
            DesktopBackground::solid_color([0.2, 0.3, 0.4, 0.1])
                .unwrap()
                .resolve(DesktopTheme::Dark),
            [0.2, 0.3, 0.4, 1.0]
        );
    }
}
