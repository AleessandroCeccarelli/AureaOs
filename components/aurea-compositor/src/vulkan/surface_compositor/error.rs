// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Error types for SHM surface composition.

use std::{error::Error, fmt};

/// Errors raised while preparing or recording a SHM-composited frame.
#[derive(Debug)]
pub(crate) enum VulkanSurfaceCompositionError {
    Pipeline(String),
    Texture(String),
    Descriptor(String),
    Command(String),
    DeviceLost(smithay::reexports::ash::vk::Result),
    InvalidSurfaceGeometry,
    TooManySurfaces,
    FrameMemoryLimitExceeded,
}

impl fmt::Display for VulkanSurfaceCompositionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pipeline(error) => {
                write!(formatter, "Surface pipeline initialization failed: {error}")
            }
            Self::Texture(error) => write!(formatter, "Wayland SHM texture upload failed: {error}"),
            Self::Descriptor(error) => write!(
                formatter,
                "Surface texture descriptor creation failed: {error}"
            ),
            Self::DeviceLost(error) => write!(formatter, "Vulkan device was lost during surface composition: {error:?}"),
            Self::Command(error) => {
                write!(formatter, "Surface composition command failed: {error}")
            }
            Self::InvalidSurfaceGeometry => {
                formatter.write_str("A visible Wayland surface has invalid physical geometry")
            }
            Self::TooManySurfaces => {
                formatter.write_str("The frame exceeds the configured visible-surface limit")
            }
            Self::FrameMemoryLimitExceeded => formatter
                .write_str("The frame exceeds the configured aggregate SHM texture memory limit"),
        }
    }
}

impl Error for VulkanSurfaceCompositionError {}
