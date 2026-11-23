// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli

//! Embedded SVG artwork for the desktop Trash.

use super::{DesktopTheme, TrashContentsState};

/// Returns the embedded SVG selected by the active theme and Trash contents.
///
/// The returned bytes are static resources compiled into Aurea Shell. The
/// presentation layer is responsible for decoding and drawing the SVG.
pub fn svg(theme: DesktopTheme, state: TrashContentsState) -> &'static [u8] {
    match (theme, state) {
        (DesktopTheme::Light, TrashContentsState::Empty) => {
            include_bytes!("../../../../aurea-human-interfaces/icons/source/system/trash/light/trash-empty.svg")
        }
        (DesktopTheme::Light, TrashContentsState::PartiallyFull) => {
            include_bytes!("../../../../aurea-human-interfaces/icons/source/system/trash/light/trash-half.svg")
        }
        (DesktopTheme::Light, TrashContentsState::Full) => {
            include_bytes!("../../../../aurea-human-interfaces/icons/source/system/trash/light/trash-full.svg")
        }
        (DesktopTheme::Dark, TrashContentsState::Empty) => {
            include_bytes!("../../../../aurea-human-interfaces/icons/source/system/trash/dark/trash-empty.svg")
        }
        (DesktopTheme::Dark, TrashContentsState::PartiallyFull) => {
            include_bytes!("../../../../aurea-human-interfaces/icons/source/system/trash/dark/trash-half.svg")
        }
        (DesktopTheme::Dark, TrashContentsState::Full) => {
            include_bytes!("../../../../aurea-human-interfaces/icons/source/system/trash/dark/trash-full.svg")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DesktopTheme, TrashContentsState, svg};

    #[test]
    fn every_theme_and_contents_state_resolves_to_an_embedded_svg() {
        for theme in [DesktopTheme::Light, DesktopTheme::Dark] {
            for state in [
                TrashContentsState::Empty,
                TrashContentsState::PartiallyFull,
                TrashContentsState::Full,
            ] {
                let asset = svg(theme, state);
                let source = std::str::from_utf8(asset).expect("SVG source must be UTF-8");
                assert!(source.contains("<svg "));
                assert!(source.contains("</svg>"));
            }
        }
    }

    #[test]
    fn light_and_dark_resolve_to_distinct_assets() {
        assert_ne!(
            svg(DesktopTheme::Light, TrashContentsState::Empty),
            svg(DesktopTheme::Dark, TrashContentsState::Empty)
        );
    }

    #[test]
    fn full_state_uses_the_approved_full_artwork() {
        let source = std::str::from_utf8(svg(
            DesktopTheme::Light,
            TrashContentsState::Full,
        ))
        .unwrap();
        assert!(source.contains("overfilled pile"));
    }
}
