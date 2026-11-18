// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Error types for Vulkan readback and staging-pool operations.

use std::{error::Error, fmt};

use smithay::reexports::ash::vk;

/// Errors returned by the offscreen readback buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanReadbackError {
    InvalidDimensions,
    SizeOverflow,
    BufferCreation(vk::Result),
    MemoryTypeUnavailable,
    MemoryAllocation(vk::Result),
    MemoryBinding(vk::Result),
    Mapping(vk::Result),
    Invalidation(vk::Result),
    UnsupportedImageFormat,
    MissingTransferSourceUsage,
    CommandBuffer(vk::Result),
    FenceCreation(vk::Result),
    Submission(vk::Result),
    SubmissionUncertain(vk::Result),
    FenceStatus(vk::Result),
    SubmissionNotComplete,
    PixelsAlreadyRead,
    ImageNotOwned,
    DeviceMismatch,
    InvalidPoolLimits,
    PoolExhausted,
    PoolBusy,
    PoolLockPoisoned,
    InvalidCoordinatorLimit,
    InFlightLimitReached,
}

impl fmt::Display for VulkanReadbackError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions => formatter.write_str("Readback dimensions must be non-zero"),
            Self::SizeOverflow => formatter.write_str("Readback byte size overflowed"),
            Self::BufferCreation(error) => {
                write!(formatter, "Readback buffer creation failed: {error:?}")
            }
            Self::MemoryTypeUnavailable => {
                formatter.write_str("No host-visible Vulkan memory type is available")
            }
            Self::MemoryAllocation(error) => {
                write!(formatter, "Readback memory allocation failed: {error:?}")
            }
            Self::MemoryBinding(error) => {
                write!(formatter, "Readback memory binding failed: {error:?}")
            }
            Self::Mapping(error) => write!(formatter, "Readback memory mapping failed: {error:?}"),
            Self::Invalidation(error) => {
                write!(formatter, "Readback memory invalidation failed: {error:?}")
            }
            Self::UnsupportedImageFormat => {
                formatter.write_str("Readback currently supports R8G8B8A8_UNORM images only")
            }
            Self::MissingTransferSourceUsage => {
                formatter.write_str("Readback image lacks TRANSFER_SRC usage")
            }
            Self::CommandBuffer(error) => write!(
                formatter,
                "Readback command-buffer operation failed: {error:?}"
            ),
            Self::FenceCreation(error) => {
                write!(formatter, "Readback fence creation failed: {error:?}")
            }
            Self::Submission(error) => {
                write!(formatter, "Readback queue submission failed: {error:?}")
            }
            Self::SubmissionUncertain(error) => write!(
                formatter,
                "Readback queue submission status is uncertain: {error:?}"
            ),
            Self::FenceStatus(error) => {
                write!(formatter, "Readback fence status query failed: {error:?}")
            }
            Self::SubmissionNotComplete => {
                formatter.write_str("Readback submission has not completed")
            }
            Self::PixelsAlreadyRead => {
                formatter.write_str("Readback pixels have already been consumed")
            }
            Self::ImageNotOwned => {
                formatter.write_str("Readback requires a view that owns its source image")
            }
            Self::DeviceMismatch => {
                formatter.write_str("Readback pool and source image use different Vulkan devices")
            }
            Self::InvalidPoolLimits => {
                formatter.write_str("Readback pool limits must be non-zero")
            }
            Self::PoolExhausted => {
                formatter.write_str("Readback staging pool capacity is exhausted")
            }
            Self::PoolBusy => {
                formatter.write_str("Readback staging pool is busy; capture can be skipped")
            }
            Self::PoolLockPoisoned => {
                formatter.write_str("Readback staging pool lock is poisoned")
            }
            Self::InvalidCoordinatorLimit => {
                formatter.write_str("Readback coordinator limit must be non-zero")
            }
            Self::InFlightLimitReached => {
                formatter.write_str("Readback coordinator has reached its in-flight limit")
            }
        }
    }
}

impl Error for VulkanReadbackError {}
