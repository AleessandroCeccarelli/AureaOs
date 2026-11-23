// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! CPU pixel-layout conversion for GBM upload buffers.

use std::{error::Error, fmt};

use smithay::backend::allocator::Fourcc;

/// Invalid source or destination layout for a CPU upload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GbmUploadLayoutError {
    InvalidDimensions,
    UnsupportedFormat(Fourcc),
    SourceSizeOverflow,
    DestinationSizeOverflow,
    SourceLengthMismatch,
    DestinationStrideTooSmall,
}

impl fmt::Display for GbmUploadLayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidDimensions => "GBM upload dimensions must be non-zero",
            Self::UnsupportedFormat(format) => {
                return write!(formatter, "GBM upload does not support {format:?}");
            }
            Self::SourceSizeOverflow => "GBM upload source size overflow",
            Self::DestinationSizeOverflow => "GBM upload destination size overflow",
            Self::SourceLengthMismatch => "RGBA8 source length does not match its dimensions",
            Self::DestinationStrideTooSmall => {
                "GBM destination stride is smaller than one RGBA8 row"
            }
        };
        formatter.write_str(message)
    }
}

impl Error for GbmUploadLayoutError {}

/// Converts tightly packed RGBA8 pixels into a DRM fourcc byte layout.
///
/// Rows are padded to the destination stride. The supported formats describe
/// little-endian memory byte order: ABGR = RGBA, ARGB/XRGB = BGRA.
pub fn pack_rgba8_for_fourcc(
    pixels: &[u8],
    width: u32,
    height: u32,
    destination_stride: usize,
    format: Fourcc,
) -> Result<Vec<u8>, GbmUploadLayoutError> {
    if width == 0 || height == 0 {
        return Err(GbmUploadLayoutError::InvalidDimensions);
    }
    if !matches!(
        format,
        Fourcc::Abgr8888 | Fourcc::Argb8888 | Fourcc::Xrgb8888
    ) {
        return Err(GbmUploadLayoutError::UnsupportedFormat(format));
    }

    let source_row_bytes = (width as usize)
        .checked_mul(4)
        .ok_or(GbmUploadLayoutError::SourceSizeOverflow)?;
    let expected_source_len = source_row_bytes
        .checked_mul(height as usize)
        .ok_or(GbmUploadLayoutError::SourceSizeOverflow)?;

    if pixels.len() != expected_source_len {
        return Err(GbmUploadLayoutError::SourceLengthMismatch);
    }
    if destination_stride < source_row_bytes {
        return Err(GbmUploadLayoutError::DestinationStrideTooSmall);
    }

    let destination_len = destination_stride
        .checked_mul(height as usize)
        .ok_or(GbmUploadLayoutError::DestinationSizeOverflow)?;
    let mut destination = vec![0; destination_len];

    for row in 0..height as usize {
        for column in 0..width as usize {
            let source = row * source_row_bytes + column * 4;
            let target = row * destination_stride + column * 4;
            let red = pixels[source];
            let green = pixels[source + 1];
            let blue = pixels[source + 2];
            let alpha = pixels[source + 3];
            let output = match format {
                Fourcc::Abgr8888 => [red, green, blue, alpha],
                Fourcc::Argb8888 => [blue, green, red, alpha],
                Fourcc::Xrgb8888 => [blue, green, red, 0xff],
                _ => unreachable!("format was validated above"),
            };
            destination[target..target + 4].copy_from_slice(&output);
        }
    }

    Ok(destination)
}

/// Copies tightly packed RGBA8 rows without changing their pixel bytes.
///
/// Padding bytes are initialized to zero. The caller remains responsible for
/// verifying the GBM buffer format, dimensions, WRITE allocation flag and
/// actual stride before invoking this.
pub fn pack_rgba8_rows(
    pixels: &[u8],
    width: u32,
    height: u32,
    destination_stride: usize,
) -> Result<Vec<u8>, GbmUploadLayoutError> {
    if width == 0 || height == 0 {
        return Err(GbmUploadLayoutError::InvalidDimensions);
    }

    let source_row_bytes = (width as usize)
        .checked_mul(4)
        .ok_or(GbmUploadLayoutError::SourceSizeOverflow)?;
    let expected_source_len = source_row_bytes
        .checked_mul(height as usize)
        .ok_or(GbmUploadLayoutError::SourceSizeOverflow)?;

    if pixels.len() != expected_source_len {
        return Err(GbmUploadLayoutError::SourceLengthMismatch);
    }
    if destination_stride < source_row_bytes {
        return Err(GbmUploadLayoutError::DestinationStrideTooSmall);
    }

    let destination_len = destination_stride
        .checked_mul(height as usize)
        .ok_or(GbmUploadLayoutError::DestinationSizeOverflow)?;
    let mut destination = vec![0; destination_len];

    for row in 0..height as usize {
        let source_start = row * source_row_bytes;
        let destination_start = row * destination_stride;
        destination[destination_start..destination_start + source_row_bytes]
            .copy_from_slice(&pixels[source_start..source_start + source_row_bytes]);
    }

    Ok(destination)
}

#[cfg(test)]
#[path = "../../tests/unit/drm/gbm_upload_layout.rs"]
mod tests;
