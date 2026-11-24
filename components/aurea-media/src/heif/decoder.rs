// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Bounded HEIC/AVIF decoding through the system libheif implementation.

use libheif_rs::{ColorSpace, DecodingOptions, HeifContext, LibHeif, RgbChroma, SecurityLimits};
use crate::{DecodeError, DecodeLimits, ImageFormat};



pub(super) fn decode(
    expected_format: ImageFormat,
    input: &[u8],
    limits: DecodeLimits,
) -> Result<(u32, u32, Vec<u8>), DecodeError> {
    if super::detect_container_format(input) != Some(expected_format) {
        return Err(DecodeError::InvalidImage);
    }

    let mut context = HeifContext::new().map_err(|_| DecodeError::InvalidImage)?;
    let mut security = SecurityLimits::new();
    security.set_max_image_size_pixels(limits.max_pixels());
    security.set_max_total_memory(u64::try_from(limits.max_output_bytes()).unwrap_or(u64::MAX));
    context.set_security_limits(&security).map_err(|_| DecodeError::InvalidImage)?;
    context.set_max_decoding_threads(1);
    context.read_bytes(input).map_err(|_| DecodeError::InvalidImage)?;

    if context.has_sequence() || context.image_ids().len() != 1 {
        return Err(DecodeError::UnsupportedFormat);
    }

    let handle = context.primary_image_handle().map_err(|_| DecodeError::InvalidImage)?;
    let width = handle.width();
    let height = handle.height();
    validate_dimensions(width, height, limits)?;

    let libheif = LibHeif::new_checked().map_err(|_| DecodeError::DecoderFailure)?;
    let mut options = DecodingOptions::new().ok_or(DecodeError::DecoderFailure)?;
    options.set_strict_decoding(true);
    options.set_convert_hdr_to_8bit(true);
    let image = libheif.decode(&handle, ColorSpace::Rgb(RgbChroma::Rgba), Some(options))
        .map_err(|_| DecodeError::InvalidImage)?;

    if image.width() != width || image.height() != height {
        return Err(DecodeError::InvalidDimensions);
    }
    let plane = image.planes().interleaved.ok_or(DecodeError::UnsupportedColorSpace)?;
    if plane.width != width || plane.height != height {
        return Err(DecodeError::InvalidDimensions);
    }

    let row_bytes = usize::try_from(width).ok().and_then(|value| value.checked_mul(4))
        .ok_or(DecodeError::InvalidDimensions)?;
    if plane.stride < row_bytes {
        return Err(DecodeError::InvalidDimensions);
    }
    let height_usize = usize::try_from(height).map_err(|_| DecodeError::InvalidDimensions)?;
    let source_len = plane.stride.checked_mul(height_usize).ok_or(DecodeError::InvalidDimensions)?;
    if plane.data.len() < source_len {
        return Err(DecodeError::InvalidImage);
    }

    let output_len = row_bytes.checked_mul(height_usize).ok_or(DecodeError::InvalidDimensions)?;
    let mut output = Vec::new();
    output.try_reserve_exact(output_len).map_err(|_| DecodeError::AllocationFailed)?;
    output.resize(output_len, 0);

    for row in 0..height_usize {
        let source_start = row.checked_mul(plane.stride).ok_or(DecodeError::InvalidDimensions)?;
        let source_end = source_start.checked_add(row_bytes).ok_or(DecodeError::InvalidDimensions)?;
        let target_start = row.checked_mul(row_bytes).ok_or(DecodeError::InvalidDimensions)?;
        let target_end = target_start.checked_add(row_bytes).ok_or(DecodeError::InvalidDimensions)?;
        output[target_start..target_end].copy_from_slice(&plane.data[source_start..source_end]);
    }
    Ok((width, height, output))
}

fn validate_dimensions(width: u32, height: u32, limits: DecodeLimits) -> Result<(), DecodeError> {
    if width == 0 || height == 0 || width > limits.max_dimension() || height > limits.max_dimension() {
        return Err(DecodeError::InvalidDimensions);
    }
    let pixels = u64::from(width).checked_mul(u64::from(height)).ok_or(DecodeError::InvalidDimensions)?;
    if pixels > limits.max_pixels() {
        return Err(DecodeError::PixelLimitExceeded);
    }
    let bytes = pixels.checked_mul(4).ok_or(DecodeError::InvalidDimensions)?;
    if bytes > limits.max_output_bytes() as u64 {
        return Err(DecodeError::OutputLimitExceeded);
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::decode;
    use crate::{DecodeError, DecodeLimits, ImageFormat};

    #[test]
    fn rejects_a_container_that_does_not_match_the_selected_format() {
        let mut input = Vec::new();
        input.extend_from_slice(&16u32.to_be_bytes());
        input.extend_from_slice(b"ftypheic");
        input.extend_from_slice(&0u32.to_be_bytes());

        assert_eq!(
            decode(ImageFormat::Avif, &input, DecodeLimits::default()).unwrap_err(),
            DecodeError::InvalidImage
        );
    }
}
