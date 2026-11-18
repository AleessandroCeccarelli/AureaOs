// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Validated snapshots of Wayland shared-memory buffers.

use std::{fmt, ptr};

use smithay::{
    reexports::wayland_server::protocol::{wl_buffer::WlBuffer, wl_shm::Format},
    wayland::shm::{BufferData, with_buffer_contents},
};

/// Maximum amount of client-controlled pixel data copied for one surface.
pub const MAX_SHM_SNAPSHOT_BYTES: usize = 256 * 1024 * 1024;

/// Pixel formats represented by an owned Wayland SHM snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShmPixelFormat {
    Argb8888,
    Xrgb8888,
    Rgba8888,
}

/// An owned, tightly packed copy of a client SHM buffer.
///
/// The copy is made while Smithay holds the SHM pool access callback. No
/// borrowed reference to client-writable memory escapes that callback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShmBufferSnapshot {
    width: u32,
    height: u32,
    format: ShmPixelFormat,
    pixels: Vec<u8>,
}

impl ShmBufferSnapshot {
    /// Copies a supported Wayland SHM buffer into compositor-owned memory.
    pub fn copy_from_wl_buffer(buffer: &WlBuffer) -> Result<Self, ShmBufferError> {
        let snapshot = with_buffer_contents(buffer, |pool, pool_len, metadata| {
            Self::copy_from_pool(pool, pool_len, metadata)
        })
        .map_err(|error| match error {
            smithay::wayland::shm::BufferAccessError::NotManaged => ShmBufferError::NotSharedMemory,
            other => ShmBufferError::BufferAccess(format!("{other:?}")),
        })?;

        snapshot
    }

    fn copy_from_pool(
        pool: *const u8,
        pool_len: usize,
        metadata: BufferData,
    ) -> Result<Self, ShmBufferError> {
        let format = match metadata.format {
            Format::Argb8888 => ShmPixelFormat::Argb8888,
            Format::Xrgb8888 => ShmPixelFormat::Xrgb8888,
            _ => return Err(ShmBufferError::UnsupportedFormat),
        };

        let layout = ValidatedLayout::new(
            metadata.offset,
            metadata.width,
            metadata.height,
            metadata.stride,
            pool_len,
        )?;

        if pool.is_null() {
            return Err(ShmBufferError::InvalidPool);
        }

        let mut pixels = vec![0; layout.packed_len];
        for row in 0..layout.height {
            let source_offset = layout.offset + row * layout.stride;
            let destination_offset = row * layout.row_bytes;

            // SAFETY: the source range was checked against pool_len above.
            // Smithay guarantees the pointer remains valid for this callback.
            // A raw copy is used deliberately: client-writable SHM is never
            // exposed as a Rust slice or reference.
            unsafe {
                ptr::copy_nonoverlapping(
                    pool.add(source_offset),
                    pixels.as_mut_ptr().add(destination_offset),
                    layout.row_bytes,
                );
            }
        }

        Ok(Self {
            width: layout.width as u32,
            height: layout.height as u32,
            format,
            pixels,
        })
    }

    /// Creates an owned RGBA8 snapshot for compositor-generated pixel data.
    pub(crate) fn from_rgba8(
        width: u32,
        height: u32,
        pixels: Vec<u8>,
    ) -> Result<Self, ShmBufferError> {
        if width == 0 || height == 0 {
            return Err(ShmBufferError::InvalidDimensions);
        }

        let expected_len = usize::try_from(width)
            .ok()
            .and_then(|width| {
                usize::try_from(height)
                    .ok()
                    .and_then(|height| width.checked_mul(height))
            })
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or(ShmBufferError::SnapshotTooLarge)?;

        if expected_len > MAX_SHM_SNAPSHOT_BYTES {
            return Err(ShmBufferError::SnapshotTooLarge);
        }
        if pixels.len() != expected_len {
            return Err(ShmBufferError::InvalidPixelData);
        }

        Ok(Self {
            width,
            height,
            format: ShmPixelFormat::Rgba8888,
            pixels,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn format(&self) -> ShmPixelFormat {
        self.format
    }

    /// Returns tightly packed bytes in the current pixel format.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Converts this owned snapshot in place to tightly packed RGBA8888.
    ///
    /// Wayland ARGB8888 and XRGB8888 are native-endian 32-bit values. On
    /// little-endian systems their byte order is BGRA and BGRX respectively.
    /// Conversion happens in the existing allocation to avoid doubling the
    /// memory used by a client-controlled SHM buffer.
    pub fn into_rgba8(mut self) -> Self {
        match self.format {
            ShmPixelFormat::Rgba8888 => return self,
            ShmPixelFormat::Argb8888 | ShmPixelFormat::Xrgb8888 => {}
        }

        let has_alpha = self.format == ShmPixelFormat::Argb8888;
        for pixel in self.pixels.chunks_exact_mut(4) {
            let value = u32::from_ne_bytes([pixel[0], pixel[1], pixel[2], pixel[3]]);
            pixel[0] = ((value >> 16) & 0xff) as u8;
            pixel[1] = ((value >> 8) & 0xff) as u8;
            pixel[2] = (value & 0xff) as u8;
            pixel[3] = if has_alpha {
                ((value >> 24) & 0xff) as u8
            } else {
                0xff
            };
        }

        self.format = ShmPixelFormat::Rgba8888;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShmBufferError {
    NotSharedMemory,
    BufferAccess(String),
    UnsupportedFormat,
    InvalidDimensions,
    InvalidStride,
    InvalidPool,
    OutOfBounds,
    SnapshotTooLarge,
    InvalidPixelData,
}

impl fmt::Display for ShmBufferError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Wayland SHM buffer error: {self:?}")
    }
}

impl std::error::Error for ShmBufferError {}

#[derive(Debug)]
struct ValidatedLayout {
    offset: usize,
    width: usize,
    height: usize,
    stride: usize,
    row_bytes: usize,
    packed_len: usize,
}

impl ValidatedLayout {
    fn new(
        offset: i32,
        width: i32,
        height: i32,
        stride: i32,
        pool_len: usize,
    ) -> Result<Self, ShmBufferError> {
        if offset < 0 || width <= 0 || height <= 0 {
            return Err(ShmBufferError::InvalidDimensions);
        }

        let offset = usize::try_from(offset).map_err(|_| ShmBufferError::InvalidDimensions)?;
        let width = usize::try_from(width).map_err(|_| ShmBufferError::InvalidDimensions)?;
        let height = usize::try_from(height).map_err(|_| ShmBufferError::InvalidDimensions)?;
        let stride = usize::try_from(stride).map_err(|_| ShmBufferError::InvalidStride)?;
        let row_bytes = width
            .checked_mul(4)
            .ok_or(ShmBufferError::SnapshotTooLarge)?;

        if stride < row_bytes {
            return Err(ShmBufferError::InvalidStride);
        }

        let packed_len = row_bytes
            .checked_mul(height)
            .ok_or(ShmBufferError::SnapshotTooLarge)?;
        if packed_len > MAX_SHM_SNAPSHOT_BYTES {
            return Err(ShmBufferError::SnapshotTooLarge);
        }

        let last_row_offset = (height - 1)
            .checked_mul(stride)
            .and_then(|value| offset.checked_add(value))
            .ok_or(ShmBufferError::OutOfBounds)?;
        let required_end = last_row_offset
            .checked_add(row_bytes)
            .ok_or(ShmBufferError::OutOfBounds)?;

        if required_end > pool_len {
            return Err(ShmBufferError::OutOfBounds);
        }

        Ok(Self {
            offset,
            width,
            height,
            stride,
            row_bytes,
            packed_len,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_tightly_packed_buffer() {
        let layout = ValidatedLayout::new(0, 4, 2, 16, 32).unwrap();
        assert_eq!(layout.row_bytes, 16);
        assert_eq!(layout.packed_len, 32);
    }

    #[test]
    fn accepts_padding_between_rows() {
        let layout = ValidatedLayout::new(8, 2, 2, 12, 32).unwrap();
        assert_eq!(layout.offset, 8);
        assert_eq!(layout.row_bytes, 8);
        assert_eq!(layout.packed_len, 16);
    }

    #[test]
    fn rejects_non_positive_dimensions_and_negative_offset() {
        assert_eq!(
            ValidatedLayout::new(0, 0, 1, 4, 4).unwrap_err(),
            ShmBufferError::InvalidDimensions
        );
        assert_eq!(
            ValidatedLayout::new(-1, 1, 1, 4, 4).unwrap_err(),
            ShmBufferError::InvalidDimensions
        );
    }

    #[test]
    fn rejects_stride_shorter_than_a_pixel_row() {
        assert_eq!(
            ValidatedLayout::new(0, 2, 1, 7, 8).unwrap_err(),
            ShmBufferError::InvalidStride
        );
    }

    #[test]
    fn rejects_pool_ranges_that_end_before_the_last_pixel() {
        assert_eq!(
            ValidatedLayout::new(4, 2, 2, 8, 19).unwrap_err(),
            ShmBufferError::OutOfBounds
        );
    }

    #[test]
    fn accepts_snapshot_at_the_memory_limit() {
        let layout = ValidatedLayout::new(0, 8192, 8192, 32768, usize::MAX)
            .expect("a snapshot exactly at the configured limit should be accepted");

        assert_eq!(layout.packed_len, MAX_SHM_SNAPSHOT_BYTES);
    }

    #[test]
    fn rejects_snapshots_above_the_memory_limit() {
        assert_eq!(
            ValidatedLayout::new(0, 8193, 8192, 32772, usize::MAX).unwrap_err(),
            ShmBufferError::SnapshotTooLarge
        );
    }

    #[test]
    fn rejects_stride_conversion_overflow_and_negative_stride() {
        assert_eq!(
            ValidatedLayout::new(0, 1, 1, -1, 4).unwrap_err(),
            ShmBufferError::InvalidStride
        );
    }

    #[test]
    fn rejects_zero_height() {
        assert_eq!(
            ValidatedLayout::new(0, 1, 0, 4, 4).unwrap_err(),
            ShmBufferError::InvalidDimensions
        );
    }

    #[test]
    fn rejects_offset_beyond_shared_memory_pool() {
        assert_eq!(
            ValidatedLayout::new(8, 1, 1, 4, 8).unwrap_err(),
            ShmBufferError::OutOfBounds
        );
    }

    #[test]
    fn rejects_stride_smaller_than_wide_pixel_row() {
        assert_eq!(
            ValidatedLayout::new(0, i32::MAX, 1, i32::MAX, usize::MAX).unwrap_err(),
            ShmBufferError::InvalidStride
        );
    }

    #[test]
    fn converts_argb8888_native_endian_pixels_to_rgba8_in_place() {
        let argb = 0x7f12_34a5_u32;
        let snapshot = ShmBufferSnapshot {
            width: 1,
            height: 1,
            format: ShmPixelFormat::Argb8888,
            pixels: argb.to_ne_bytes().to_vec(),
        };

        let converted = snapshot.into_rgba8();

        assert_eq!(converted.format(), ShmPixelFormat::Rgba8888);
        assert_eq!(converted.pixels(), &[0x12, 0x34, 0xa5, 0x7f]);
        assert_eq!(converted.pixels().len(), 4);
    }

    #[test]
    fn converts_xrgb8888_to_opaque_rgba8() {
        let xrgb = 0x0012_34a5_u32;
        let snapshot = ShmBufferSnapshot {
            width: 1,
            height: 1,
            format: ShmPixelFormat::Xrgb8888,
            pixels: xrgb.to_ne_bytes().to_vec(),
        };

        let converted = snapshot.into_rgba8();

        assert_eq!(converted.format(), ShmPixelFormat::Rgba8888);
        assert_eq!(converted.pixels(), &[0x12, 0x34, 0xa5, 0xff]);
    }

    #[test]
    fn rgba8_constructor_rejects_inconsistent_pixel_data() {
        assert_eq!(
            ShmBufferSnapshot::from_rgba8(2, 1, vec![0; 4]).unwrap_err(),
            ShmBufferError::InvalidPixelData
        );
    }

    #[test]
    fn rgba8_constructor_rejects_zero_dimensions() {
        assert_eq!(
            ShmBufferSnapshot::from_rgba8(0, 1, Vec::new()).unwrap_err(),
            ShmBufferError::InvalidDimensions
        );
    }

    #[test]
    fn rgba8_conversion_is_idempotent() {
        let snapshot = ShmBufferSnapshot {
            width: 1,
            height: 1,
            format: ShmPixelFormat::Rgba8888,
            pixels: vec![0x12, 0x34, 0xa5, 0x7f],
        };

        let converted = snapshot.into_rgba8();

        assert_eq!(converted.pixels(), &[0x12, 0x34, 0xa5, 0x7f]);
        assert_eq!(converted.format(), ShmPixelFormat::Rgba8888);
    }
}
