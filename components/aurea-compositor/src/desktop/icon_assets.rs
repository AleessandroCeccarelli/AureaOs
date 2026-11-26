// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Maps desktop model entries to approved, pre-rasterized icon assets.

use aurea_desktop_api::desktop::{DesktopItem, DesktopItemKind, DesktopTrashState};

use crate::{
    render::shm_buffer::{ShmBufferError, ShmBufferSnapshot},
};

use super::DesktopTheme;

/// Stable identity for one rasterized desktop artwork variant.
///
/// Revision changes when approved pixel content changes; pixel size distinguishes
/// scale-specific variants. This key belongs to the desktop model boundary, not Vulkan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct DesktopIconTextureKey {
    pub(crate) asset_id: &'static str,
    pub(crate) revision: u64,
    pub(crate) pixel_size: u32,
}

/// Raster sizes generated for a 64-logical-pixel desktop icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DesktopIconRasterSize {
    OneX,
    OneAndHalfX,
    TwoX,
    ThreeX,
}

impl DesktopIconRasterSize {
    /// Chooses the smallest approved raster that does not undersample the output.
    pub(crate) fn for_scale(scale: f64) -> Self {
        if !scale.is_finite() || scale <= 1.0 {
            Self::OneX
        } else if scale <= 1.5 {
            Self::OneAndHalfX
        } else if scale <= 2.0 {
            Self::TwoX
        } else {
            Self::ThreeX
        }
    }

    pub(crate) const fn pixels(self) -> u32 {
        match self {
            Self::OneX => 64,
            Self::OneAndHalfX => 96,
            Self::TwoX => 128,
            Self::ThreeX => 192,
        }
    }
}

/// A canonical generated asset and the identity used by the Vulkan cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DesktopIconAssetDescriptor {
    pub(crate) key: DesktopIconTextureKey,
    pub(crate) relative_path: String,
}

impl DesktopIconAssetDescriptor {
    /// Validates generated RGBA8 bytes and creates the cache request snapshot.
    pub(crate) fn snapshot(
        &self,
        rgba8: &[u8],
    ) -> Result<(DesktopIconTextureKey, ShmBufferSnapshot), ShmBufferError> {
        let snapshot = ShmBufferSnapshot::from_rgba8(
            self.key.pixel_size,
            self.key.pixel_size,
            rgba8.to_vec(),
        )?;
        Ok((self.key, snapshot))
    }
}

/// Resolves the pre-rasterized desktop context-menu artwork for the active theme.
pub(crate) fn desktop_context_menu_asset(
    theme: DesktopTheme,
) -> DesktopIconAssetDescriptor {
    let (theme_name, asset_id) = match theme {
        DesktopTheme::Light => ("light", "system:context-menu:light"),
        DesktopTheme::Dark => ("dark", "system:context-menu:dark"),
    };

    DesktopIconAssetDescriptor {
        key: DesktopIconTextureKey {
            asset_id,
            revision: 1,
            pixel_size: 256,
        },
        relative_path: format!(
            "icons/generated/context-menu/context-menu-{theme_name}-256.rgba"
        ),
    }
}

/// Resolves the approved Trash artwork for the current appearance and contents.
///
/// Other item kinds deliberately return no asset until their canonical artwork
/// has been approved and added to the generated asset pipeline.
pub(crate) fn desktop_icon_asset(
    item: &DesktopItem,
    theme: DesktopTheme,
    trash_contents: DesktopTrashState,
    output_scale: f64,
) -> Option<DesktopIconAssetDescriptor> {
    if !matches!(item.kind(), DesktopItemKind::Trash) {
        return None;
    }

    let (theme_name, state_name, asset_id) = match (theme, trash_contents) {
        (DesktopTheme::Light, DesktopTrashState::Empty) => {
            ("light", "empty", "system:trash:light:empty")
        }
        (DesktopTheme::Light, DesktopTrashState::PartiallyFull) => {
            ("light", "half", "system:trash:light:half")
        }
        (DesktopTheme::Light, DesktopTrashState::Full) => {
            ("light", "full", "system:trash:light:full")
        }
        (DesktopTheme::Dark, DesktopTrashState::Empty) => {
            ("dark", "empty", "system:trash:dark:empty")
        }
        (DesktopTheme::Dark, DesktopTrashState::PartiallyFull) => {
            ("dark", "half", "system:trash:dark:half")
        }
        (DesktopTheme::Dark, DesktopTrashState::Full) => {
            ("dark", "full", "system:trash:dark:full")
        }
    };

    let pixel_size = DesktopIconRasterSize::for_scale(output_scale).pixels();
    Some(DesktopIconAssetDescriptor {
        key: DesktopIconTextureKey {
            asset_id,
            revision: 1,
            pixel_size,
        },
        relative_path: format!(
            "icons/generated/trash/trash-{theme_name}-{state_name}-{pixel_size}.rgba"
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        DesktopIconRasterSize, DesktopTrashState, desktop_icon_asset,
    };
    use crate::desktop::DesktopTheme;
    use aurea_desktop_api::desktop::DesktopManager;

    #[test]
    fn resolves_context_menu_artwork_from_the_shared_human_interfaces_library() {
        let light = super::desktop_context_menu_asset(DesktopTheme::Light);
        let dark = super::desktop_context_menu_asset(DesktopTheme::Dark);

        assert_eq!(light.key.asset_id, "system:context-menu:light");
        assert_eq!(dark.key.asset_id, "system:context-menu:dark");
        assert_eq!(light.key.pixel_size, 256);
        assert_eq!(dark.key.pixel_size, 256);
        assert!(light.relative_path.ends_with("context-menu-light-256.rgba"));
        assert!(dark.relative_path.ends_with("context-menu-dark-256.rgba"));
    }

    #[test]
    fn resolves_every_approved_theme_and_trash_state() {
        let manager = DesktopManager::default();
        let trash = &manager.items()[0];

        for theme in [DesktopTheme::Light, DesktopTheme::Dark] {
            for state in [
                DesktopTrashState::Empty,
                DesktopTrashState::PartiallyFull,
                DesktopTrashState::Full,
            ] {
                let asset = desktop_icon_asset(
                    trash,
                    theme,
                    state,
                    1.0,
                )
                .expect("Trash artwork must resolve");
                assert!(asset.relative_path.ends_with("-64.rgba"));
                assert_eq!(asset.key.pixel_size, 64);
                assert_eq!(asset.key.revision, 1);
            }
        }
    }

    #[test]
    fn selects_the_requested_raster_density() {
        let manager = DesktopManager::default();
        let trash = &manager.items()[0];
        let asset = desktop_icon_asset(
            trash,
            DesktopTheme::Dark,
            DesktopTrashState::Empty,
            3.0,
        )
        .unwrap();

        assert_eq!(asset.key.pixel_size, 192);
        assert!(asset.relative_path.ends_with("trash-dark-empty-192.rgba"));
    }

    #[test]
    fn selects_the_smallest_raster_that_covers_fractional_output_scale() {
        assert_eq!(DesktopIconRasterSize::for_scale(1.0), DesktopIconRasterSize::OneX);
        assert_eq!(DesktopIconRasterSize::for_scale(1.25), DesktopIconRasterSize::OneAndHalfX);
        assert_eq!(DesktopIconRasterSize::for_scale(1.5), DesktopIconRasterSize::OneAndHalfX);
        assert_eq!(DesktopIconRasterSize::for_scale(1.75), DesktopIconRasterSize::TwoX);
        assert_eq!(DesktopIconRasterSize::for_scale(2.0), DesktopIconRasterSize::TwoX);
        assert_eq!(DesktopIconRasterSize::for_scale(2.5), DesktopIconRasterSize::ThreeX);
        assert_eq!(DesktopIconRasterSize::for_scale(f64::NAN), DesktopIconRasterSize::OneX);
    }

    #[test]
    fn snapshot_validation_rejects_incorrect_rgba_byte_count() {
        let manager = DesktopManager::default();
        let asset = desktop_icon_asset(
            &manager.items()[0],
            DesktopTheme::Light,
            DesktopTrashState::Empty,
            1.0,
        )
        .unwrap();

        assert!(asset.snapshot(&[0; 4]).is_err());
    }
}
