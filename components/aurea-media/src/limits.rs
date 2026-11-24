// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

/// Default limits for one decoded image.
pub const DEFAULT_DECODE_LIMITS: DecodeLimits =
    DecodeLimits::new(64 * 1024 * 1024, 16_384, 64 * 1024 * 1024, 256 * 1024 * 1024);

/// Explicit resource limits for one image decode request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodeLimits {
    max_input_bytes: usize,
    max_dimension: u32,
    max_pixels: u64,
    max_output_bytes: usize,
}

impl DecodeLimits {
    pub const fn new(
        max_input_bytes: usize,
        max_dimension: u32,
        max_pixels: u64,
        max_output_bytes: usize,
    ) -> Self {
        Self {
            max_input_bytes,
            max_dimension,
            max_pixels,
            max_output_bytes,
        }
    }

    pub const fn max_input_bytes(self) -> usize {
        self.max_input_bytes
    }

    pub const fn max_dimension(self) -> u32 {
        self.max_dimension
    }

    pub const fn max_pixels(self) -> u64 {
        self.max_pixels
    }

    pub const fn max_output_bytes(self) -> usize {
        self.max_output_bytes
    }
}

impl Default for DecodeLimits {
    fn default() -> Self {
        DEFAULT_DECODE_LIMITS
    }
}

#[cfg(test)]
mod tests {
    use super::{DecodeLimits, DEFAULT_DECODE_LIMITS};

    #[test]
    fn defaults_match_the_initial_security_policy() {
        assert_eq!(DEFAULT_DECODE_LIMITS.max_input_bytes(), 64 * 1024 * 1024);
        assert_eq!(DEFAULT_DECODE_LIMITS.max_dimension(), 16_384);
        assert_eq!(DEFAULT_DECODE_LIMITS.max_pixels(), 64 * 1024 * 1024);
        assert_eq!(
            DEFAULT_DECODE_LIMITS.max_output_bytes(),
            256 * 1024 * 1024
        );
    }

    #[test]
    fn custom_limits_are_preserved() {
        let limits = DecodeLimits::new(1024, 100, 5000, 20_000);
        assert_eq!(limits.max_input_bytes(), 1024);
        assert_eq!(limits.max_dimension(), 100);
        assert_eq!(limits.max_pixels(), 5000);
        assert_eq!(limits.max_output_bytes(), 20_000);
    }
}
