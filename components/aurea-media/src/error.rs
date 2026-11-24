// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use std::{error::Error, fmt};

/// Stable errors returned by the media service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    EmptyInput,
    InputTooLarge,
    UnsupportedFormat,
    InvalidImage,
    UnsupportedColorSpace,
    InvalidDimensions,
    PixelLimitExceeded,
    OutputLimitExceeded,
    AllocationFailed,
    DecoderFailure,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::EmptyInput => "image input is empty",
            Self::InputTooLarge => "compressed image exceeds the input limit",
            Self::UnsupportedFormat => "image format is not supported",
            Self::InvalidImage => "image data is invalid or truncated",
            Self::UnsupportedColorSpace => "image color space is not supported",
            Self::InvalidDimensions => "decoded image dimensions are invalid",
            Self::PixelLimitExceeded => "decoded image exceeds the pixel-count limit",
            Self::OutputLimitExceeded => "decoded image exceeds the output-memory limit",
            Self::AllocationFailed => "unable to allocate decoded image memory",
            Self::DecoderFailure => "image decoder failed",
        })
    }
}

impl Error for DecodeError {}
