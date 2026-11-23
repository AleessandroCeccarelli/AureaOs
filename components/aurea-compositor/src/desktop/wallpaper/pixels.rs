// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Bounded, owned RGBA8 pixels ready for wallpaper texture upload.

use std::{error::Error, fmt};

/// Maximum decoded wallpaper memory retained by one CPU pixel buffer.
pub const MAX_WALLPAPER_RGBA_BYTES: usize = 256 * 1024 * 1024;

/// A fully decoded, tightly packed RGBA8 wallpaper image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WallpaperPixels {
    width: u32,
    height: u32,
    rgba8: Vec<u8>,
}

/// Validation failures for decoded wallpaper pixel data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallpaperPixelsError {
    ZeroDimensions,
    DimensionsOverflow,
    MemoryLimitExceeded,
    InvalidByteLength,
}

impl fmt::Display for WallpaperPixelsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroDimensions => formatter.write_str("wallpaper dimensions must be non-zero"),
            Self::DimensionsOverflow => formatter.write_str("wallpaper pixel dimensions overflow"),
            Self::MemoryLimitExceeded => {
                write!(formatter, "decoded wallpaper exceeds the {MAX_WALLPAPER_RGBA_BYTES}-byte memory limit")
            }
            Self::InvalidByteLength => {
                formatter.write_str("RGBA8 buffer length does not match wallpaper dimensions")
            }
        }
    }
}

impl Error for WallpaperPixelsError {}

impl WallpaperPixels {
    /// Takes ownership of tightly packed RGBA8 pixels after validating size and memory use.
    pub fn new(width: u32, height: u32, rgba8: Vec<u8>) -> Result<Self, WallpaperPixelsError> {
        let expected_bytes = Self::required_byte_len(width, height)?;
        if rgba8.len() != expected_bytes {
            return Err(WallpaperPixelsError::InvalidByteLength);
        }

        Ok(Self { width, height, rgba8 })
    }

    /// Computes the exact RGBA8 allocation size with overflow and memory-limit checks.
    pub fn required_byte_len(width: u32, height: u32) -> Result<usize, WallpaperPixelsError> {
        if width == 0 || height == 0 {
            return Err(WallpaperPixelsError::ZeroDimensions);
        }

        let byte_len = u64::from(width)
            .checked_mul(u64::from(height))
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or(WallpaperPixelsError::DimensionsOverflow)?;

        let byte_len =
            usize::try_from(byte_len).map_err(|_| WallpaperPixelsError::DimensionsOverflow)?;

        if byte_len > MAX_WALLPAPER_RGBA_BYTES {
            return Err(WallpaperPixelsError::MemoryLimitExceeded);
        }

        Ok(byte_len)
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub fn rgba8(&self) -> &[u8] {
        &self.rgba8
    }

    pub fn into_rgba8(self) -> Vec<u8> {
        self.rgba8
    }

    pub fn byte_len(&self) -> usize {
        self.rgba8.len()
    }
}

#[cfg(test)]
mod tests {
    use super::{WallpaperPixels, WallpaperPixelsError, MAX_WALLPAPER_RGBA_BYTES};

    #[test]
    fn accepts_exact_rgba8_buffer() {
        let pixels = WallpaperPixels::new(2, 1, vec![0, 1, 2, 3, 4, 5, 6, 7]).unwrap();

        assert_eq!(pixels.width(), 2);
        assert_eq!(pixels.height(), 1);
        assert_eq!(pixels.byte_len(), 8);
    }

    #[test]
    fn rejects_wrong_buffer_length() {
        assert_eq!(
            WallpaperPixels::new(2, 1, vec![0; 7]).unwrap_err(),
            WallpaperPixelsError::InvalidByteLength
        );
    }

    #[test]
    fn rejects_zero_dimensions() {
        assert_eq!(
            WallpaperPixels::required_byte_len(0, 10).unwrap_err(),
            WallpaperPixelsError::ZeroDimensions
        );
    }

    #[test]
    fn rejects_images_over_the_memory_budget() {
        let width = u32::try_from(MAX_WALLPAPER_RGBA_BYTES / 4 + 1).unwrap();

        assert_eq!(
            WallpaperPixels::required_byte_len(width, 1).unwrap_err(),
            WallpaperPixelsError::MemoryLimitExceeded
        );
    }

    #[test]
    fn rejects_dimension_arithmetic_overflow() {
        assert_eq!(
            WallpaperPixels::required_byte_len(u32::MAX, u32::MAX).unwrap_err(),
            WallpaperPixelsError::DimensionsOverflow
        );
    }
}
