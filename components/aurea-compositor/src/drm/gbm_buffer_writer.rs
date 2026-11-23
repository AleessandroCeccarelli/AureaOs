// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! CPU upload into GBM buffers allocated with the WRITE usage flag.

use std::{error::Error, fmt};

use smithay::backend::allocator::{Fourcc, gbm::GbmBuffer};

use super::gbm_upload_layout::{GbmUploadLayoutError, pack_rgba8_for_fourcc};

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
    /// Converts RGBA readback pixels to the selected DRM format's byte order.
    /// XRGB receives an opaque alpha byte; ARGB and ABGR preserve source alpha.
    pub(crate) fn write_rgba8(
        &mut self,
        pixels: &[u8],
        width: u32,
        height: u32,
    ) -> Result<(), GbmBufferWriterError> {
        let format = self.buffer.format();
        if !matches!(
            format,
            Fourcc::Abgr8888 | Fourcc::Xrgb8888 | Fourcc::Argb8888
        ) {
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

        let packed =
            pack_rgba8_for_fourcc(pixels, width, height, self.buffer.stride() as usize, format)?;

        self.buffer
            .write(&packed)
            .map_err(GbmBufferWriterError::Write)
    }
}
