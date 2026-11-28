// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! JPEG decode orchestration; native ABI details live in the C shim.

use std::ffi::{c_int, c_uchar, c_ulong};

use crate::{DecodeError, DecodeLimits};

const SHIM_INVALID_INPUT: c_int = 1;
const SHIM_UNSUPPORTED_COLOR_SPACE: c_int = 2;
const SHIM_DECODER_FAILURE: c_int = 3;
const SHIM_OUTPUT_MISMATCH: c_int = 4;

unsafe extern "C" {
    fn aurea_turbojpeg_header(
        input: *const c_uchar,
        input_len: c_ulong,
        width: *mut c_int,
        height: *mut c_int,
    ) -> c_int;

    fn aurea_turbojpeg_decode(
        input: *const c_uchar,
        input_len: c_ulong,
        width: c_int,
        height: c_int,
        output: *mut c_uchar,
        output_len: usize,
    ) -> c_int;
}

pub(super) fn decode(
    input: &[u8],
    limits: DecodeLimits,
) -> Result<(u32, u32, Vec<u8>), DecodeError> {
    let input_len = c_ulong::try_from(input.len()).map_err(|_| DecodeError::InputTooLarge)?;
    let mut width = 0;
    let mut height = 0;

    // The shim reads the borrowed input and writes only the two output integers.
    let status =
        unsafe { aurea_turbojpeg_header(input.as_ptr(), input_len, &mut width, &mut height) };
    map_status(status)?;

    let width = u32::try_from(width).map_err(|_| DecodeError::InvalidDimensions)?;
    let height = u32::try_from(height).map_err(|_| DecodeError::InvalidDimensions)?;
    validate_dimensions(width, height, limits)?;

    let byte_len = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(DecodeError::InvalidDimensions)?;
    let byte_len = usize::try_from(byte_len).map_err(|_| DecodeError::InvalidDimensions)?;

    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(byte_len)
        .map_err(|_| DecodeError::AllocationFailed)?;
    pixels.resize(byte_len, 0);

    // The shim writes exactly byte_len bytes and retains no pointers after return.
    let status = unsafe {
        aurea_turbojpeg_decode(
            input.as_ptr(),
            input_len,
            width as c_int,
            height as c_int,
            pixels.as_mut_ptr(),
            pixels.len(),
        )
    };
    map_status(status)?;

    Ok((width, height, pixels))
}

fn validate_dimensions(width: u32, height: u32, limits: DecodeLimits) -> Result<(), DecodeError> {
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

    let bytes = pixels
        .checked_mul(4)
        .ok_or(DecodeError::InvalidDimensions)?;
    if bytes > limits.max_output_bytes() as u64 {
        return Err(DecodeError::OutputLimitExceeded);
    }

    Ok(())
}

fn map_status(status: c_int) -> Result<(), DecodeError> {
    match status {
        0 => Ok(()),
        SHIM_INVALID_INPUT => Err(DecodeError::InvalidImage),
        SHIM_UNSUPPORTED_COLOR_SPACE => Err(DecodeError::UnsupportedColorSpace),
        SHIM_DECODER_FAILURE => Err(DecodeError::DecoderFailure),
        SHIM_OUTPUT_MISMATCH => Err(DecodeError::InvalidDimensions),
        _ => Err(DecodeError::DecoderFailure),
    }
}

#[cfg(test)]
mod tests {
    use super::validate_dimensions;
    use crate::{DecodeError, DecodeLimits};

    #[test]
    fn rejects_axis_limit() {
        let limits = DecodeLimits::new(1024, 10, 100, 400);
        assert_eq!(
            validate_dimensions(11, 1, limits).unwrap_err(),
            DecodeError::InvalidDimensions
        );
    }

    #[test]
    fn rejects_pixel_limit() {
        let limits = DecodeLimits::new(1024, 100, 9, 400);
        assert_eq!(
            validate_dimensions(4, 3, limits).unwrap_err(),
            DecodeError::PixelLimitExceeded
        );
    }

    #[test]
    fn rejects_output_limit() {
        let limits = DecodeLimits::new(1024, 100, 100, 47);
        assert_eq!(
            validate_dimensions(4, 3, limits).unwrap_err(),
            DecodeError::OutputLimitExceeded
        );
    }

    #[test]
    fn accepts_exact_limits() {
        let limits = DecodeLimits::new(1024, 4, 12, 48);
        assert!(validate_dimensions(4, 3, limits).is_ok());
    }
}

#[cfg(test)]
mod native_tests {
    use super::decode;
    use crate::{DecodeError, DecodeLimits};

    const DEFAULT_WALLPAPER: &[u8] = include_bytes!(
        "../../../aurea-human-interfaces/resources/wallpapers/aureaos-blue-flow.jpg"
    );

    #[test]
    fn decodes_the_committed_wallpaper_to_bounded_rgba8() {
        let (width, height, pixels) = decode(DEFAULT_WALLPAPER, DecodeLimits::default()).unwrap();

        assert_eq!((width, height), (3840, 2160));
        assert_eq!(pixels.len(), 33_177_600);
        assert!(pixels.chunks_exact(4).all(|pixel| pixel[3] == 255));
    }

    #[test]
    fn rejects_truncated_jpeg_data() {
        assert_eq!(
            decode(&[0xFF, 0xD8, 0xFF], DecodeLimits::default()).unwrap_err(),
            DecodeError::InvalidImage
        );
    }

    #[test]
    fn rejects_dimensions_over_configured_limit_before_allocation() {
        let limits = DecodeLimits::new(64 * 1024 * 1024, 100, 64 * 1024 * 1024, 256 * 1024 * 1024);

        assert_eq!(
            decode(DEFAULT_WALLPAPER, limits).unwrap_err(),
            DecodeError::InvalidDimensions
        );
    }
}
