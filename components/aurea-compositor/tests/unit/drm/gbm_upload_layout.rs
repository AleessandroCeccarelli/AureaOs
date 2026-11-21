// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use crate::drm::gbm_upload_layout::{
    GbmUploadLayoutError, pack_rgba8_for_fourcc, pack_rgba8_rows,
};
use smithay::backend::allocator::Fourcc;

#[test]
fn packs_tightly_packed_rows_without_changing_pixels() {
    let pixels = [1, 2, 3, 4, 5, 6, 7, 8];
    let packed = pack_rgba8_rows(&pixels, 1, 2, 4).unwrap();

    assert_eq!(packed, pixels);
}

#[test]
fn inserts_zeroed_padding_between_rows() {
    let pixels = [1, 2, 3, 4, 5, 6, 7, 8];
    let packed = pack_rgba8_rows(&pixels, 1, 2, 8).unwrap();

    assert_eq!(packed, [1, 2, 3, 4, 0, 0, 0, 0, 5, 6, 7, 8, 0, 0, 0, 0]);
}

#[test]
fn rejects_zero_dimensions() {
    assert_eq!(
        pack_rgba8_rows(&[], 0, 1, 0),
        Err(GbmUploadLayoutError::InvalidDimensions)
    );
}

#[test]
fn rejects_incorrect_source_length() {
    assert_eq!(
        pack_rgba8_rows(&[1, 2, 3], 1, 1, 4),
        Err(GbmUploadLayoutError::SourceLengthMismatch)
    );
}

#[test]
fn rejects_destination_stride_smaller_than_pixel_row() {
    assert_eq!(
        pack_rgba8_rows(&[1, 2, 3, 4], 1, 1, 3),
        Err(GbmUploadLayoutError::DestinationStrideTooSmall)
    );
}

#[test]
fn rejects_source_size_overflow() {
    assert_eq!(
        pack_rgba8_rows(&[], u32::MAX, u32::MAX, usize::MAX),
        Err(GbmUploadLayoutError::SourceSizeOverflow)
    );
}

#[test]
fn converts_rgba_to_abgr_memory_order_without_channel_swapping() {
    let packed = pack_rgba8_for_fourcc(&[10, 20, 30, 40], 1, 1, 4, Fourcc::Abgr8888).unwrap();

    assert_eq!(packed, [10, 20, 30, 40]);
}

#[test]
fn converts_rgba_to_argb_memory_order() {
    let packed = pack_rgba8_for_fourcc(&[10, 20, 30, 40], 1, 1, 4, Fourcc::Argb8888).unwrap();

    assert_eq!(packed, [30, 20, 10, 40]);
}

#[test]
fn converts_rgba_to_opaque_xrgb_memory_order() {
    let packed = pack_rgba8_for_fourcc(&[10, 20, 30, 40], 1, 1, 4, Fourcc::Xrgb8888).unwrap();

    assert_eq!(packed, [30, 20, 10, 255]);
}

#[test]
fn converts_each_pixel_and_preserves_row_padding() {
    let pixels = [10, 20, 30, 40, 50, 60, 70, 80];
    let packed =
        pack_rgba8_for_fourcc(&pixels, 1, 2, 8, Fourcc::Argb8888).unwrap();

    assert_eq!(packed, [30, 20, 10, 40, 0, 0, 0, 0, 70, 60, 50, 80, 0, 0, 0, 0]);
}

#[test]
fn rejects_unsupported_drm_format() {
    assert_eq!(
        pack_rgba8_for_fourcc(&[1, 2, 3, 4], 1, 1, 4, Fourcc::Rgb565),
        Err(GbmUploadLayoutError::UnsupportedFormat(Fourcc::Rgb565))
    );
}
