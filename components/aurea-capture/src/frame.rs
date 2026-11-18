// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Capture Core
// Copyright (C) 2026 Alessandro Ceccarelli

//! Owned video-frame data shared by capture backends and media encoders.

use crate::CaptureFrameError;

/// Pixel byte order produced by a capture backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapturePixelFormat {
    Rgba8,
    Bgra8,
}

impl CapturePixelFormat {
    pub const fn bytes_per_pixel(self) -> usize {
        4
    }
}

/// Monotonic presentation timestamp in nanoseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CaptureTimestamp(u64);

impl CaptureTimestamp {
    pub const fn from_nanos(value: u64) -> Self {
        Self(value)
    }

    pub const fn as_nanos(self) -> u64 {
        self.0
    }
}

/// An owned, tightly packed frame ready for image or video processing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureFrame {
    width: u32,
    height: u32,
    stride: usize,
    format: CapturePixelFormat,
    timestamp: CaptureTimestamp,
    pixels: Vec<u8>,
}

impl CaptureFrame {
    /// Creates a frame after validating dimensions, stride and byte length.
    pub fn new(
        width: u32,
        height: u32,
        stride: usize,
        format: CapturePixelFormat,
        timestamp: CaptureTimestamp,
        pixels: Vec<u8>,
    ) -> Result<Self, CaptureFrameError> {
        if width == 0 || height == 0 {
            return Err(CaptureFrameError::InvalidDimensions);
        }

        let minimum_stride = (width as usize)
            .checked_mul(format.bytes_per_pixel())
            .ok_or(CaptureFrameError::SizeOverflow)?;
        if stride < minimum_stride {
            return Err(CaptureFrameError::InvalidStride);
        }

        let expected_len = stride
            .checked_mul(height as usize)
            .ok_or(CaptureFrameError::SizeOverflow)?;
        if pixels.len() != expected_len {
            return Err(CaptureFrameError::InvalidBufferLength);
        }

        Ok(Self {
            width,
            height,
            stride,
            format,
            timestamp,
            pixels,
        })
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub const fn stride(&self) -> usize {
        self.stride
    }

    pub const fn format(&self) -> CapturePixelFormat {
        self.format
    }

    pub const fn timestamp(&self) -> CaptureTimestamp {
        self.timestamp
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn byte_len(&self) -> usize {
        self.pixels.len()
    }
}
