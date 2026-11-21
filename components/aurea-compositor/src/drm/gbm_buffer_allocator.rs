// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Allocates GBM buffers for the CPU compatibility presentation path.

use std::{error::Error, fmt};

use smithay::backend::allocator::{
    Fourcc,
    gbm::{GbmBuffer, GbmBufferFlags, GbmDevice},
};

use crate::drm::gbm_buffer_writer::GbmCpuUploadBuffer;

/// Errors returned by the GBM CPU-upload allocator.
#[derive(Debug)]
pub(crate) enum GbmCpuUploadAllocationError {
    InvalidDimensions,
    UnsupportedFormatUsage {
        format: Fourcc,
    },
    Allocation(String),
    UnexpectedBufferFormat {
        expected: Fourcc,
        actual: Fourcc,
    },
    UnexpectedBufferDimensions {
        expected_width: u32,
        expected_height: u32,
        actual_width: u32,
        actual_height: u32,
    },
}

impl fmt::Display for GbmCpuUploadAllocationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions => {
                formatter.write_str("GBM CPU-upload dimensions must be non-zero")
            }
            Self::UnsupportedFormatUsage { format } => write!(
                formatter,
                "GBM device does not support {format:?} with WRITE and SCANOUT usage"
            ),
            Self::Allocation(message) => {
                write!(
                    formatter,
                    "GBM CPU-upload buffer allocation failed: {message}"
                )
            }
            Self::UnexpectedBufferFormat { expected, actual } => {
                write!(formatter, "GBM allocated {actual:?}, expected {expected:?}")
            }
            Self::UnexpectedBufferDimensions {
                expected_width,
                expected_height,
                actual_width,
                actual_height,
            } => write!(
                formatter,
                "GBM allocated {actual_width}x{actual_height}, expected {expected_width}x{expected_height}"
            ),
        }
    }
}

impl Error for GbmCpuUploadAllocationError {}

/// Allocates only buffers that can be written by the CPU and scanned out by KMS.
///
/// The allocator deliberately requests the single format currently supported
/// by the RGBA8 readback writer. Other DRM formats need explicit conversion.
pub(crate) struct GbmCpuUploadAllocator<'device, T: std::os::fd::AsFd> {
    device: &'device GbmDevice<T>,
}

impl<'device, T: std::os::fd::AsFd> GbmCpuUploadAllocator<'device, T> {
    /// Borrows an already initialized GBM device.
    pub(crate) fn new(device: &'device GbmDevice<T>) -> Self {
        Self { device }
    }

    /// Allocates a WRITE|SCANOUT buffer in a format supported by the KMS plane.
    pub(crate) fn allocate(
        &self,
        width: u32,
        height: u32,
        plane_formats: &[Fourcc],
    ) -> Result<GbmCpuUploadBuffer, GbmCpuUploadAllocationError> {
        if width == 0 || height == 0 {
            return Err(GbmCpuUploadAllocationError::InvalidDimensions);
        }

        let usage = GbmBufferFlags::WRITE | GbmBufferFlags::SCANOUT;
        let format = [Fourcc::Xrgb8888, Fourcc::Argb8888, Fourcc::Abgr8888]
            .into_iter()
            .find(|format| {
                plane_formats.contains(format) && self.device.is_format_supported(*format, usage)
            })
            .ok_or_else(|| {
                let format = plane_formats
                    .iter()
                    .copied()
                    .find(|format| matches!(format, Fourcc::Xrgb8888 | Fourcc::Argb8888 | Fourcc::Abgr8888))
                    .unwrap_or(Fourcc::Xrgb8888);
                GbmCpuUploadAllocationError::UnsupportedFormatUsage { format }
            })?;

        let buffer_object = self
            .device
            .create_buffer_object::<()>(width, height, format, usage)
            .map_err(|error| GbmCpuUploadAllocationError::Allocation(error.to_string()))?;
        let buffer = GbmBuffer::from_bo(buffer_object, false);

        if buffer.format() != format {
            return Err(GbmCpuUploadAllocationError::UnexpectedBufferFormat {
                expected: format,
                actual: buffer.format(),
            });
        }
        if buffer.width() != width || buffer.height() != height {
            return Err(GbmCpuUploadAllocationError::UnexpectedBufferDimensions {
                expected_width: width,
                expected_height: height,
                actual_width: buffer.width(),
                actual_height: buffer.height(),
            });
        }

        Ok(GbmCpuUploadBuffer::from_write_allocated(buffer))
    }
}
