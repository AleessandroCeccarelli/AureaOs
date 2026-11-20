// SPDX-License-Identifier: GPL-3.0-or-later
// AureaOS — Color Management

//! Little CMS implementation of the backend-independent transform contract.

use lcms2::{Flags, Intent, PixelFormat, Profile, Transform};

use crate::{
    ColorTransformEngine, ColorTransformError, ColorTransformOptions, RenderingIntent,
};

/// Executes RGB ICC transforms through Little CMS.
///
/// A transform and its profile handles are created per call. No mutable
/// Little CMS cache or transform handle is shared between threads.
#[derive(Debug, Default, Clone, Copy)]
pub struct LittleCmsTransformEngine;

impl ColorTransformEngine for LittleCmsTransformEngine {
    fn transform_rgba_f32(
        &self,
        source_profile: &[u8],
        destination_profile: &[u8],
        pixels: &mut [f32],
        pixel_count: usize,
        options: ColorTransformOptions,
    ) -> Result<(), ColorTransformError> {
        let expected_components = pixel_count
            .checked_mul(4)
            .ok_or(ColorTransformError::InvalidPixelBuffer)?;
        if pixels.len() != expected_components || pixels.iter().any(|value| !value.is_finite()) {
            return Err(ColorTransformError::InvalidPixelBuffer);
        }
        if pixel_count == 0 {
            return Ok(());
        }
        if pixel_count > u32::MAX as usize {
            return Err(ColorTransformError::InvalidPixelBuffer);
        }
        if !is_rgb_profile(source_profile) || !is_rgb_profile(destination_profile) {
            return Err(ColorTransformError::UnsupportedTransform);
        }

        let source = Profile::new_icc(source_profile)
            .map_err(|_| ColorTransformError::EngineFailure)?;
        let destination = Profile::new_icc(destination_profile)
            .map_err(|_| ColorTransformError::EngineFailure)?;

        let intent = match options.rendering_intent {
            RenderingIntent::Perceptual => Intent::Perceptual,
            RenderingIntent::RelativeColorimetric => Intent::RelativeColorimetric,
            RenderingIntent::Saturation => Intent::Saturation,
            RenderingIntent::AbsoluteColorimetric => Intent::AbsoluteColorimetric,
        };

        let flags = if options.black_point_compensation {
            Flags::COPY_ALPHA | Flags::BLACKPOINT_COMPENSATION
        } else {
            Flags::COPY_ALPHA
        };

        let transform = Transform::new_flags(
            &source,
            PixelFormat::RGBA_FLT,
            &destination,
            PixelFormat::RGBA_FLT,
            intent,
            flags,
        )
        .map_err(|_| ColorTransformError::UnsupportedTransform)?;

        let mut rgba_pixels: Vec<[f32; 4]> = pixels
            .chunks_exact(4)
            .map(|rgba| [rgba[0], rgba[1], rgba[2], rgba[3]])
            .collect();

        transform.transform_in_place(&mut rgba_pixels);

        for (target, transformed) in pixels.chunks_exact_mut(4).zip(rgba_pixels) {
            target.copy_from_slice(&transformed);
        }

        Ok(())
    }
}

fn is_rgb_profile(profile: &[u8]) -> bool {
    profile.get(16..20) == Some(b"RGB ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transform_rgba_f32;

    fn srgb_profile() -> Vec<u8> {
        Profile::new_srgb().icc().expect("serialize sRGB profile")
    }

    #[test]
    fn srgb_to_srgb_preserves_neutral_values_and_alpha() {
        let profile = srgb_profile();
        let mut pixels = [0.25_f32, 0.5, 0.75, 0.625];

        transform_rgba_f32(
            &LittleCmsTransformEngine,
            &profile,
            &profile,
            &mut pixels,
            1,
            ColorTransformOptions::default(),
        )
        .expect("sRGB transform should succeed");

        for (actual, expected) in pixels[..3].iter().zip([0.25_f32, 0.5, 0.75]) {
            assert!((actual - expected).abs() < 0.0005);
        }
        assert_eq!(pixels[3], 0.625);
    }

    #[test]
    fn rejects_non_rgb_profiles() {
        let profile = srgb_profile();
        let mut non_rgb = profile.clone();
        non_rgb[16..20].copy_from_slice(b"CMYK");
        let mut pixels = [0.0_f32; 4];

        assert_eq!(
            LittleCmsTransformEngine.transform_rgba_f32(
                &non_rgb,
                &profile,
                &mut pixels,
                1,
                ColorTransformOptions::default(),
            ),
            Err(ColorTransformError::UnsupportedTransform)
        );
    }

    #[test]
    fn rejects_invalid_direct_engine_buffer() {
        let profile = srgb_profile();
        let mut pixels = [0.0_f32; 3];

        assert_eq!(
            LittleCmsTransformEngine.transform_rgba_f32(
                &profile,
                &profile,
                &mut pixels,
                1,
                ColorTransformOptions::default(),
            ),
            Err(ColorTransformError::InvalidPixelBuffer)
        );
    }

    #[test]
    fn empty_buffer_does_not_create_a_transform() {
        assert_eq!(
            LittleCmsTransformEngine.transform_rgba_f32(
                &[],
                &[],
                &mut [],
                0,
                ColorTransformOptions::default(),
            ),
            Ok(())
        );
    }
}
