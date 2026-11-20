// SPDX-License-Identifier: GPL-3.0-or-later
// AureaOS — Color Management

//! Backend-independent contract for profile-to-profile pixel transforms.

use crate::{IccHeaderError, inspect_profile};

/// ICC rendering intent used when mapping colors between profile gamuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderingIntent {
    Perceptual,
    RelativeColorimetric,
    Saturation,
    AbsoluteColorimetric,
}

/// Options that affect the ICC transform, not the source profile bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorTransformOptions {
    pub rendering_intent: RenderingIntent,
    pub black_point_compensation: bool,
}

impl Default for ColorTransformOptions {
    fn default() -> Self {
        Self {
            rendering_intent: RenderingIntent::RelativeColorimetric,
            black_point_compensation: true,
        }
    }
}

/// Errors raised before or during a profile-to-profile transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorTransformError {
    InvalidSourceProfile(IccHeaderError),
    InvalidDestinationProfile(IccHeaderError),
    InvalidPixelBuffer,
    UnsupportedTransform,
    EngineFailure,
}

/// Executes color transforms using validated ICC profile data.
///
/// Pixels are interleaved RGBA float samples. Their transfer function is
/// defined by the caller's pipeline stage; the engine must not silently
/// assume one. Values outside 0..=1 are permitted for extended-range/HDR
/// workflows. Alpha must remain unchanged.
pub trait ColorTransformEngine: Send + Sync {
    /// Transforms pixel_count RGBA pixels in place.
    ///
    /// The slice must contain exactly pixel_count * 4 finite components.
    fn transform_rgba_f32(
        &self,
        source_profile: &[u8],
        destination_profile: &[u8],
        pixels: &mut [f32],
        pixel_count: usize,
        options: ColorTransformOptions,
    ) -> Result<(), ColorTransformError>;
}

/// Validates profile payloads and the pixel buffer before calling an engine.
pub fn transform_rgba_f32(
    engine: &dyn ColorTransformEngine,
    source_profile: &[u8],
    destination_profile: &[u8],
    pixels: &mut [f32],
    pixel_count: usize,
    options: ColorTransformOptions,
) -> Result<(), ColorTransformError> {
    inspect_profile(source_profile).map_err(ColorTransformError::InvalidSourceProfile)?;
    inspect_profile(destination_profile).map_err(ColorTransformError::InvalidDestinationProfile)?;

    let expected_components = pixel_count
        .checked_mul(4)
        .ok_or(ColorTransformError::InvalidPixelBuffer)?;
    if pixels.len() != expected_components || pixels.iter().any(|value| !value.is_finite()) {
        return Err(ColorTransformError::InvalidPixelBuffer);
    }

    engine.transform_rgba_f32(
        source_profile,
        destination_profile,
        pixels,
        pixel_count,
        options,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    struct IdentityEngine;

    impl ColorTransformEngine for IdentityEngine {
        fn transform_rgba_f32(
            &self,
            _source_profile: &[u8],
            _destination_profile: &[u8],
            _pixels: &mut [f32],
            _pixel_count: usize,
            _options: ColorTransformOptions,
        ) -> Result<(), ColorTransformError> {
            Ok(())
        }
    }

    fn profile() -> Vec<u8> {
        let mut bytes = vec![0_u8; 132];
        bytes[0..4].copy_from_slice(&132_u32.to_be_bytes());
        bytes[8..12].copy_from_slice(b"\x04\x30\x00\x00");
        bytes[12..16].copy_from_slice(b"mntr");
        bytes[16..20].copy_from_slice(b"RGB ");
        bytes[20..24].copy_from_slice(b"XYZ ");
        bytes[36..40].copy_from_slice(b"acsp");
        bytes
    }

    #[test]
    fn rejects_invalid_profile_before_calling_engine() {
        let mut pixels = [0.0_f32; 4];
        assert!(matches!(
            transform_rgba_f32(
                &IdentityEngine,
                b"invalid",
                &profile(),
                &mut pixels,
                1,
                ColorTransformOptions::default(),
            ),
            Err(ColorTransformError::InvalidSourceProfile(_))
        ));
    }

    #[test]
    fn rejects_pixel_count_mismatch() {
        let mut pixels = [0.0_f32; 3];
        assert_eq!(
            transform_rgba_f32(
                &IdentityEngine,
                &profile(),
                &profile(),
                &mut pixels,
                1,
                ColorTransformOptions::default(),
            ),
            Err(ColorTransformError::InvalidPixelBuffer)
        );
    }

    #[test]
    fn rejects_non_finite_pixel_components() {
        let mut pixels = [0.0_f32, f32::NAN, 0.0, 1.0];
        assert_eq!(
            transform_rgba_f32(
                &IdentityEngine,
                &profile(),
                &profile(),
                &mut pixels,
                1,
                ColorTransformOptions::default(),
            ),
            Err(ColorTransformError::InvalidPixelBuffer)
        );
    }

    #[test]
    fn default_options_use_relative_colorimetric_with_bpc() {
        assert_eq!(
            ColorTransformOptions::default(),
            ColorTransformOptions {
                rendering_intent: RenderingIntent::RelativeColorimetric,
                black_point_compensation: true,
            }
        );
    }
}
