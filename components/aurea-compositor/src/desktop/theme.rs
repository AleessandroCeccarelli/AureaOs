// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Desktop appearance mode and canonical desktop surface colors.

/// Appearance mode used by the compositor desktop surface.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DesktopTheme {
    /// AureaOS light desktop token (#edf4ff).
    #[default]
    Light,
    /// AureaOS dark desktop token (#0d1320).
    Dark,
}

impl DesktopTheme {
    /// Returns the canonical desktop color as normalized RGBA channels.
    pub const fn desktop_color(self) -> [f32; 4] {
        match self {
            Self::Light => [0.9294, 0.9569, 1.0, 1.0],
            Self::Dark => [0.0510, 0.0745, 0.1255, 1.0],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DesktopTheme;

    #[test]
    fn light_and_dark_tokens_are_opaque() {
        assert_eq!(DesktopTheme::Light.desktop_color()[3], 1.0);
        assert_eq!(DesktopTheme::Dark.desktop_color()[3], 1.0);
        assert_ne!(DesktopTheme::Light.desktop_color(), DesktopTheme::Dark.desktop_color());
    }
}
