// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use crate::{heif, jpeg, raster, DecodeError, DecodeLimits};

/// Encoded still-image format accepted by the media service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Bmp,
    Tiff,
    WebP,
    Heic,
    Avif,
}

/// Pixel layout returned by the media service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Rgba8,
}

/// Owned, tightly packed decoded image pixels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedImage {
    width: u32,
    height: u32,
    format: PixelFormat,
    pixels: Vec<u8>,
}

impl DecodedImage {
    fn new(width: u32, height: u32, pixels: Vec<u8>) -> Self {
        Self { width, height, format: PixelFormat::Rgba8, pixels }
    }

    pub const fn width(&self) -> u32 { self.width }
    pub const fn height(&self) -> u32 { self.height }
    pub const fn format(&self) -> PixelFormat { self.format }
    pub fn pixels(&self) -> &[u8] { &self.pixels }
    pub fn byte_len(&self) -> usize { self.pixels.len() }
    pub fn into_pixels(self) -> Vec<u8> { self.pixels }
}

/// Decodes a still image synchronously under explicit limits.
///
/// Call from a bounded worker, never from a compositor/rendering thread.
pub fn decode(
    format: ImageFormat,
    input: &[u8],
    limits: DecodeLimits,
) -> Result<DecodedImage, DecodeError> {
    if input.is_empty() { return Err(DecodeError::EmptyInput); }
    if input.len() > limits.max_input_bytes() { return Err(DecodeError::InputTooLarge); }

    let decoded = match format {
        ImageFormat::Jpeg => jpeg::decode(input, limits),
        ImageFormat::Png | ImageFormat::Bmp | ImageFormat::Tiff | ImageFormat::WebP => {
            raster::decode(format, input, limits)
        }
        ImageFormat::Heic | ImageFormat::Avif => {
            #[cfg(feature = "heif")]
            { heif::decode(input, limits) }
            #[cfg(not(feature = "heif"))]
            { let _ = (input, limits); Err(DecodeError::UnsupportedFormat) }
        }
    }?;
    Ok(DecodedImage::new(decoded.0, decoded.1, decoded.2))
}

#[cfg(test)]
mod tests {
    use super::{decode, ImageFormat};
    use crate::{DecodeError, DecodeLimits};

    #[test]
    fn rejects_empty_input_before_backend() {
        assert_eq!(decode(ImageFormat::Jpeg, &[], DecodeLimits::default()).unwrap_err(), DecodeError::EmptyInput);
    }

    #[test]
    fn rejects_input_over_limit() {
        let limits = DecodeLimits::new(3, 100, 100, 400);
        assert_eq!(decode(ImageFormat::Jpeg, &[1, 2, 3, 4], limits).unwrap_err(), DecodeError::InputTooLarge);
    }

    #[cfg(not(feature = "heif"))]
    #[test]
    fn heif_is_explicitly_unsupported_without_feature() {
        assert_eq!(decode(ImageFormat::Heic, b"file", DecodeLimits::default()).unwrap_err(), DecodeError::UnsupportedFormat);
    }
}
