// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Bounded decoding for PNG, BMP, GIF, TIFF and WebP raster images.

use std::io::Cursor;

use image::{ImageReader, Limits};

use crate::{DecodeError, DecodeLimits, ImageFormat};

pub(super) fn decode(
    format: ImageFormat,
    input: &[u8],
    limits: DecodeLimits,
) -> Result<(u32, u32, Vec<u8>), DecodeError> {
    let image_format = to_backend_format(format)?;
    let mut reader = ImageReader::with_format(Cursor::new(input), image_format);
    reader.limits(backend_limits(limits));

    let image = reader.decode().map_err(map_decode_error)?;
    let width = image.width();
    let height = image.height();
    validate_dimensions(width, height, limits)?;

    let pixels = image.into_rgba8().into_raw();
    let expected_len = expected_output_len(width, height)?;
    if pixels.len() != expected_len {
        return Err(DecodeError::InvalidDimensions);
    }

    Ok((width, height, pixels))
}

fn to_backend_format(format: ImageFormat) -> Result<image::ImageFormat, DecodeError> {
    match format {
        ImageFormat::Png => Ok(image::ImageFormat::Png),
        ImageFormat::Bmp => Ok(image::ImageFormat::Bmp),
        ImageFormat::Gif => Ok(image::ImageFormat::Gif),
        ImageFormat::Tiff => Ok(image::ImageFormat::Tiff),
        ImageFormat::WebP => Ok(image::ImageFormat::WebP),
        ImageFormat::Jpeg => Err(DecodeError::UnsupportedFormat),
    }
}

fn backend_limits(limits: DecodeLimits) -> Limits {
    Limits {
        max_image_width: Some(limits.max_dimension()),
        max_image_height: Some(limits.max_dimension()),
        max_alloc: Some(limits.max_output_bytes() as u64),
        ..Limits::default()
    }
}

fn validate_dimensions(
    width: u32,
    height: u32,
    limits: DecodeLimits,
) -> Result<(), DecodeError> {
    if width == 0
        || height == 0
        || width > limits.max_dimension()
        || height > limits.max_dimension()
    {
        return Err(DecodeError::InvalidDimensions);
    }

    let pixels = u64::from(width)
        .checked_mul(u64::from(height))
        .ok_or(DecodeError::InvalidDimensions)?;
    if pixels > limits.max_pixels() {
        return Err(DecodeError::PixelLimitExceeded);
    }

    let output_bytes = pixels
        .checked_mul(4)
        .ok_or(DecodeError::InvalidDimensions)?;
    if output_bytes > limits.max_output_bytes() as u64 {
        return Err(DecodeError::OutputLimitExceeded);
    }

    Ok(())
}

fn expected_output_len(width: u32, height: u32) -> Result<usize, DecodeError> {
    let bytes = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(DecodeError::InvalidDimensions)?;
    usize::try_from(bytes).map_err(|_| DecodeError::InvalidDimensions)
}

fn map_decode_error(error: image::ImageError) -> DecodeError {
    match error {
        image::ImageError::Limits(_) => DecodeError::OutputLimitExceeded,
        image::ImageError::Unsupported(_) => DecodeError::UnsupportedFormat,
        image::ImageError::Decoding(_) | image::ImageError::IoError(_) => DecodeError::InvalidImage,
        image::ImageError::Encoding(_) => DecodeError::DecoderFailure,
    }
}
