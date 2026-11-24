// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Owned and validated wallpaper image assets.

use std::{
    error::Error,
    fmt,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

use aurea_media::DecodeLimits;

const DEFAULT_WALLPAPER_BYTES: &[u8] = include_bytes!(
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../aurea-human-interfaces/resources/wallpapers/aureaos-midnight-flow.jpg"
    )
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallpaperFormat {
    Jpeg,
    Png,
    Bmp,
    Tiff,
    WebP,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallpaperDimensions {
    width: u32,
    height: u32,
}

impl WallpaperDimensions {
    pub const fn width(self) -> u32 { self.width }
    pub const fn height(self) -> u32 { self.height }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WallpaperAssetError {
    EmptyId,
    EmptyInput,
    InputTooLarge,
    UnsupportedFormat,
    Io(String),
}

impl fmt::Display for WallpaperAssetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId => formatter.write_str("wallpaper asset identifier is empty"),
            Self::EmptyInput => formatter.write_str("wallpaper image is empty"),
            Self::InputTooLarge => formatter.write_str("wallpaper image exceeds the input limit"),
            Self::UnsupportedFormat => formatter.write_str("wallpaper image format is unsupported"),
            Self::Io(message) => write!(formatter, "wallpaper file read failed: {message}"),
        }
    }
}

impl Error for WallpaperAssetError {}

/// Compressed wallpaper bytes owned independently of the source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WallpaperAsset {
    id: Arc<str>,
    format: WallpaperFormat,
    bytes: Arc<[u8]>,
    dimensions: Option<WallpaperDimensions>,
}

impl WallpaperAsset {
    pub fn default_wallpaper() -> Option<Self> {
        Self::from_bytes("aureaos-midnight-flow", DEFAULT_WALLPAPER_BYTES.to_vec()).ok()
    }

    /// Creates an owned asset from bytes. Format is identified by file signature,
    /// never by a filename extension.
    pub fn from_bytes(
        id: impl Into<Arc<str>>,
        bytes: Vec<u8>,
    ) -> Result<Self, WallpaperAssetError> {
        let id = id.into();
        if id.is_empty() {
            return Err(WallpaperAssetError::EmptyId);
        }
        if bytes.is_empty() {
            return Err(WallpaperAssetError::EmptyInput);
        }
        if bytes.len() > DecodeLimits::default().max_input_bytes() {
            return Err(WallpaperAssetError::InputTooLarge);
        }

        let format = detect_format(&bytes).ok_or(WallpaperAssetError::UnsupportedFormat)?;
        let dimensions = match format {
            WallpaperFormat::Jpeg => jpeg_dimensions(&bytes),
            _ => None,
        };

        Ok(Self {
            id,
            format,
            bytes: Arc::from(bytes),
            dimensions,
        })
    }

    /// Reads a user-selected file into owned memory. Call from a settings or
    /// media-I/O task, never from the compositor frame/render loop.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, WallpaperAssetError> {
        let path = path.as_ref();
        let metadata = std::fs::metadata(path)
            .map_err(|error| WallpaperAssetError::Io(error.to_string()))?;
        let max_bytes = DecodeLimits::default().max_input_bytes() as u64;
        if metadata.len() == 0 {
            return Err(WallpaperAssetError::EmptyInput);
        }
        if metadata.len() > max_bytes {
            return Err(WallpaperAssetError::InputTooLarge);
        }

        let mut file = File::open(path)
            .map_err(|error| WallpaperAssetError::Io(error.to_string()))?;
        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        file.take(max_bytes + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| WallpaperAssetError::Io(error.to_string()))?;
        if bytes.len() as u64 > max_bytes {
            return Err(WallpaperAssetError::InputTooLarge);
        }

        let id = stable_file_id(path);
        Self::from_bytes(id, bytes)
    }

    pub fn id(&self) -> &str { &self.id }
    pub const fn format(&self) -> WallpaperFormat { self.format }
    pub fn bytes(&self) -> &[u8] { &self.bytes }
    pub const fn dimensions(&self) -> Option<WallpaperDimensions> { self.dimensions }
}

fn stable_file_id(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| PathBuf::from(path))
        .to_string_lossy()
        .into_owned()
}

fn detect_format(bytes: &[u8]) -> Option<WallpaperFormat> {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some(WallpaperFormat::Jpeg);
    }
    if bytes.starts_with(b"\x89PNG\r\n\x1A\n") {
        return Some(WallpaperFormat::Png);
    }
    if bytes.starts_with(b"BM") {
        return Some(WallpaperFormat::Bmp);
    }
    if bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*") {
        return Some(WallpaperFormat::Tiff);
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some(WallpaperFormat::WebP);
    }
    None
}

/// Reads JPEG SOF metadata without decoding or allocating the pixel buffer.
fn jpeg_dimensions(bytes: &[u8]) -> Option<WallpaperDimensions> {
    if bytes.get(0..2)? != [0xFF, 0xD8] { return None; }
    let mut offset = 2usize;
    while offset < bytes.len() {
        if bytes.get(offset) != Some(&0xFF) { return None; }
        while bytes.get(offset) == Some(&0xFF) {
            offset = offset.checked_add(1)?;
        }
        let marker = *bytes.get(offset)?;
        offset = offset.checked_add(1)?;
        if marker == 0xD9 || marker == 0xDA { return None; }
        if marker == 0x01 || (0xD0..=0xD7).contains(&marker) { continue; }

        let segment_length = usize::from(u16::from_be_bytes([
            *bytes.get(offset)?,
            *bytes.get(offset.checked_add(1)?)?,
        ]));
        if segment_length < 2 { return None; }
        let segment_start = offset.checked_add(2)?;
        let segment_end = offset.checked_add(segment_length)?;
        if segment_end > bytes.len() { return None; }

        if matches!(marker, 0xC0..=0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF) {
            if segment_length < 8 { return None; }
            let components = usize::from(*bytes.get(segment_start.checked_add(5)?)?);
            if components == 0 || segment_length != 8usize.checked_add(components.checked_mul(3)?)? {
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
            if width == 0 || height == 0 { return None; }
            return Some(WallpaperDimensions { width: u32::from(width), height: u32::from(height) });
        }
        offset = segment_end;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{WallpaperAsset, WallpaperAssetError, WallpaperFormat, detect_format};

    #[test]
    fn default_wallpaper_is_owned_and_identified_as_jpeg() {
        let wallpaper = WallpaperAsset::default_wallpaper().unwrap();
        assert_eq!(wallpaper.id(), "aureaos-midnight-flow");
        assert_eq!(wallpaper.format(), WallpaperFormat::Jpeg);
        assert!(wallpaper.bytes().len() > 1024);
        assert!(wallpaper.dimensions().is_some());
    }

    #[test]
    fn detects_supported_formats_from_signatures() {
        assert_eq!(detect_format(b"\x89PNG\r\n\x1A\n"), Some(WallpaperFormat::Png));
        assert_eq!(detect_format(b"BM"), Some(WallpaperFormat::Bmp));
        assert_eq!(detect_format(b"II*\0"), Some(WallpaperFormat::Tiff));
        assert_eq!(detect_format(b"RIFF0000WEBP"), Some(WallpaperFormat::WebP));
    }

    #[test]
    fn rejects_empty_and_unknown_inputs() {
        assert_eq!(
            WallpaperAsset::from_bytes("test", Vec::new()).unwrap_err(),
            WallpaperAssetError::EmptyInput
        );
        assert_eq!(
            WallpaperAsset::from_bytes("test", b"unknown".to_vec()).unwrap_err(),
            WallpaperAssetError::UnsupportedFormat
        );
    }

    #[test]
    fn rejects_empty_identifiers() {
        assert_eq!(
            WallpaperAsset::from_bytes("", b"BM".to_vec()).unwrap_err(),
            WallpaperAssetError::EmptyId
        );
    }
}
