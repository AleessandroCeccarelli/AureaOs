// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Validated metadata for an embedded desktop wallpaper.

const DEFAULT_WALLPAPER_BYTES: &[u8] = include_bytes!(
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../aurea-human-interfaces/resources/wallpapers/aureaos-midnight-flow.jpg"
    )
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallpaperFormat {
    Jpeg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallpaperDimensions {
    width: u16,
    height: u16,
}

impl WallpaperDimensions {
    pub const fn width(self) -> u16 {
        self.width
    }

    pub const fn height(self) -> u16 {
        self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallpaperAsset {
    id: &'static str,
    format: WallpaperFormat,
    bytes: &'static [u8],
    dimensions: WallpaperDimensions,
}

impl WallpaperAsset {
    /// Returns the selected AureaOS default wallpaper when its JPEG header is valid.
    pub fn default_wallpaper() -> Option<Self> {
        let dimensions = jpeg_dimensions(DEFAULT_WALLPAPER_BYTES)?;
        Some(Self {
            id: "aureaos-midnight-flow",
            format: WallpaperFormat::Jpeg,
            bytes: DEFAULT_WALLPAPER_BYTES,
            dimensions,
        })
    }

    pub const fn id(self) -> &'static str {
        self.id
    }

    pub const fn format(self) -> WallpaperFormat {
        self.format
    }

    /// Returns the original compressed JPEG bytes embedded in the compositor.
    pub const fn bytes(self) -> &'static [u8] {
        self.bytes
    }

    pub const fn dimensions(self) -> WallpaperDimensions {
        self.dimensions
    }
}

/// Reads JPEG SOF metadata without decoding or allocating the pixel buffer.
fn jpeg_dimensions(bytes: &[u8]) -> Option<WallpaperDimensions> {
    if bytes.get(0..2)? != [0xFF, 0xD8] {
        return None;
    }

    let mut offset = 2usize;
    while offset < bytes.len() {
        if bytes.get(offset) != Some(&0xFF) {
            return None;
        }
        while bytes.get(offset) == Some(&0xFF) {
            offset = offset.checked_add(1)?;
        }

        let marker = *bytes.get(offset)?;
        offset = offset.checked_add(1)?;

        if marker == 0xD9 || marker == 0xDA {
            return None;
        }
        if marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
            continue;
        }

        let segment_length = usize::from(u16::from_be_bytes([
            *bytes.get(offset)?,
            *bytes.get(offset.checked_add(1)?)?,
        ]));
        if segment_length < 2 {
            return None;
        }

        let segment_start = offset.checked_add(2)?;
        let segment_end = offset.checked_add(segment_length)?;
        if segment_end > bytes.len() {
            return None;
        }

        if matches!(marker, 0xC0..=0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF) {
            // A JPEG SOF payload contains precision, dimensions, component
            // count, then exactly three bytes for each component.
            if segment_length < 8 {
                return None;
            }

            let component_count = usize::from(*bytes.get(segment_start.checked_add(5)?)?);
            let expected_length = 8usize.checked_add(component_count.checked_mul(3)?)?;
            if component_count == 0 || segment_length != expected_length {
                return None;
            }

            let height = u16::from_be_bytes([
                *bytes.get(segment_start.checked_add(1)?)?,
                *bytes.get(segment_start.checked_add(2)?)?,
            ]);
            let width = u16::from_be_bytes([
                *bytes.get(segment_start.checked_add(3)?)?,
                *bytes.get(segment_start.checked_add(4)?)?,
            ]);
            if width == 0 || height == 0 {
                return None;
            }
            return Some(WallpaperDimensions { width, height });
        }

        offset = segment_end;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{WallpaperAsset, jpeg_dimensions};

    #[test]
    fn selected_default_wallpaper_has_a_valid_jpeg_frame() {
        let wallpaper = WallpaperAsset::default_wallpaper()
            .expect("the committed default wallpaper must have a valid JPEG frame");
        assert_eq!(wallpaper.id(), "aureaos-midnight-flow");
        assert!(wallpaper.bytes().len() > 1024);
        assert!(wallpaper.dimensions().width() > 0);
        assert!(wallpaper.dimensions().height() > 0);
    }

    #[test]
    fn jpeg_header_parser_rejects_invalid_and_truncated_data() {
        assert!(jpeg_dimensions(b"not a jpeg").is_none());
        assert!(jpeg_dimensions(&[0xFF, 0xD8, 0xFF]).is_none());
    }

    #[test]
    fn jpeg_header_parser_requires_a_well_formed_frame_component_table() {
        // SOI, SOF0 (8-bit, 1x1, one component), then a minimal component table.
        let valid = [
            0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01, 0x00, 0x01, 0x01,
            0x01, 0x11, 0x00,
        ];
        assert_eq!(
            jpeg_dimensions(&valid),
            Some(super::WallpaperDimensions { width: 1, height: 1 })
        );

        // The declared component count is one, but the SOF segment is too short.
        let truncated_components = [
            0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x08, 0x08, 0x00, 0x01, 0x00, 0x01, 0x01,
        ];
        assert!(jpeg_dimensions(&truncated_components).is_none());

        // A zero component count cannot describe a decodable frame.
        let zero_components = [
            0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x08, 0x08, 0x00, 0x01, 0x00, 0x01, 0x00,
        ];
        assert!(jpeg_dimensions(&zero_components).is_none());
    }
}
