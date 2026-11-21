// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! CPU upload into GBM buffers allocated with the WRITE usage flag.

use std::{error::Error, fmt};

use smithay::backend::allocator::{Fourcc, gbm::GbmBuffer};

use super::gbm_upload_layout::GbmUploadLayoutError;

/// Errors returned while uploading a CPU frame into a GBM buffer.
#[derive(Debug)]
pub enum GbmBufferWriterError {
    UnsupportedFormat(Fourcc),
    DimensionMismatch {
        expected_width: u32,
        expected_height: u32,
        actual_width: u32,
        actual_height: u32,
    },
    Layout(GbmUploadLayoutError),
    Write(std::io::Error),
}

impl fmt::Display for GbmBufferWriterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFormat(format) => {
                write!(formatter, "GBM CPU upload does not support {format:?}")
            }
            Self::DimensionMismatch {
                expected_width,
                expected_height,
                actual_width,
                actual_height,
            } => write!(
                formatter,
                "GBM upload dimensions {expected_width}x{expected_height} do not match buffer {actual_width}x{actual_height}"
            ),
            Self::Layout(error) => write!(formatter, "Invalid GBM upload layout: {error}"),
            Self::Write(error) => write!(formatter, "GBM buffer CPU write failed: {error}"),
        }
    }
}

impl Error for GbmBufferWriterError {}

impl From<GbmUploadLayoutError> for GbmBufferWriterError {
    fn from(error: GbmUploadLayoutError) -> Self {
        Self::Layout(error)
    }
}

/// A GBM buffer whose allocator guaranteed the WRITE usage flag.
///
/// Construction is crate-private so the future allocator can be the only
/// production path that establishes this invariant.
pub(crate) struct GbmCpuUploadBuffer {
    buffer: GbmBuffer,
}

impl GbmCpuUploadBuffer {
    /// Wraps a buffer returned by an allocator that requested WRITE.
    pub(crate) fn from_write_allocated(buffer: GbmBuffer) -> Self {
        Self { buffer }
    }

    /// Borrows the underlying GBM allocation for KMS framebuffer registration.
    pub(crate) fn gbm_buffer(&self) -> &GbmBuffer {
        &self.buffer
    }

    /// Copies tightly packed RGBA8 pixels into the GBM buffer.
    ///
    /// DRM_FORMAT_ABGR8888 is used because its byte order in memory on
    /// little-endian systems is R, G, B, A. Other formats require an explicit
    /// conversion and are deliberately rejected.
    pub(crate) fn write_rgba8(
        &mut self,
        pixels: &[u8],
        width: u32,
        height: u32,
    ) -> Result<(), GbmBufferWriterError> {
        let format = self.buffer.format();
        if !matches!(format, Fourcc::Abgr8888 | Fourcc::Xrgb8888 | Fourcc::Argb8888) {
            return Err(GbmBufferWriterError::UnsupportedFormat(format));
        }

        let actual_width = self.buffer.width();
        let actual_height = self.buffer.height();
        if actual_width != width || actual_height != height {
            return Err(GbmBufferWriterError::DimensionMismatch {
                expected_width: width,
                expected_height: height,
                actual_width,
                actual_height,
            });
        }

        let stride = self.buffer.stride() as usize;
        let row_bytes = (width as usize)
            .checked_mul(4)
            .ok_or(GbmUploadLayoutError::SourceSizeOverflow)?;
        let expected_len = row_bytes
            .checked_mul(height as usize)
            .ok_or(GbmUploadLayoutError::SourceSizeOverflow)?;
        if pixels.len() != expected_len {
            return Err(GbmUploadLayoutError::SourceLengthMismatch.into());
        }
        if stride < row_bytes {
            return Err(GbmUploadLayoutError::DestinationStrideTooSmall.into());
        }
        let destination_len = stride
            .checked_mul(height as usize)
            .ok_or(GbmUploadLayoutError::DestinationSizeOverflow)?;
        let mut packed = vec![0; destination_len];

        for y in 0..height as usize {
            for x in 0..width as usize {
                let source = y * row_bytes + x * 4;
                let destination = y * stride + x * 4;
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
                packed[destination..destination + 4].copy_from_slice(&output);
            }
        }

        self.buffer
            .write(&packed)
            .map_err(GbmBufferWriterError::Write)
    }
}
