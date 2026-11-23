// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Validated metadata for an embedded desktop wallpaper.

/// The default wallpaper is embedded in the compositor binary so the desktop
/// does not depend on the current working directory or a mounted data volume.
const DEFAULT_WALLPAPER_BYTES: &[u8] = include_bytes!(
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../aurea-human-interfaces/resources/wallpapers/aureaos-midnight-flow.jpg"
    )
);

/// Supported wallpaper container format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallpaperFormat {
    Jpeg,
}

/// Pixel dimensions read from the JPEG frame header.
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

/// An embedded wallpaper whose JPEG frame dimensions have been validated.
#[derive(Debug, Clone, Copy)]
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
        while bytes.get(offset) == Some(&0xFF) {
            offset = offset.checked_add(1)?;
        }

        let marker = *bytes.get(offset)?;
        offset = offset.checked_add(1)?;

        // Standalone markers do not carry a segment length.
        if marker == 0xD9 || marker == 0xDA {
            return None;
        }
        if marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
            continue;
        }

        if bytes.get(offset) != Some(&0x00) && marker != 0x00 {
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

            // Baseline, progressive and the other DCT SOF markers.
            if matches!(marker, 0xC0..=0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF) {
                if segment_length < 8 {
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
}
