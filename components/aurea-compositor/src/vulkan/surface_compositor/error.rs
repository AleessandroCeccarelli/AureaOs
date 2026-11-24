// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Error types for SHM surface composition.

use std::{error::Error, fmt};

use smithay::reexports::ash::vk;

use crate::vulkan::queue_submitter::VulkanQueueSubmitError;

/// Errors raised while preparing or recording a SHM-composited frame.
#[derive(Debug)]
pub(crate) enum VulkanSurfaceCompositionError {
    Pipeline(String),
    Texture(String),
    Descriptor(String),
    Command(String),
    DeviceLost(vk::Result),
    InvalidSurfaceGeometry,
    TooManySurfaces,
    FrameMemoryLimitExceeded,
}

impl VulkanSurfaceCompositionError {
    pub(crate) fn from_shm_texture_error(
        error: super::super::VulkanShmTextureError,
    ) -> Self {
        match error {
            super::super::VulkanShmTextureError::Command(vk::Result::ERROR_DEVICE_LOST) => {
                Self::DeviceLost(vk::Result::ERROR_DEVICE_LOST)
            }
            other => Self::Texture(other.to_string()),
        }
    }

    pub(crate) fn from_surface_texture_cache_error(
        error: super::super::surface_texture_cache::VulkanSurfaceTextureCacheError,
    ) -> Self {
        match error {
            super::super::surface_texture_cache::VulkanSurfaceTextureCacheError::Texture(error) => {
                Self::from_shm_texture_error(error)
            }
            other => Self::Texture(other.to_string()),
        }
    }

    pub(crate) fn from_queue_submit_error(error: VulkanQueueSubmitError) -> Self {
        let device_lost = match error {
            VulkanQueueSubmitError::Rejected(result)
            | VulkanQueueSubmitError::SubmissionUncertain(result) => {
                result == vk::Result::ERROR_DEVICE_LOST
            }
            VulkanQueueSubmitError::CompletionUncertain { wait_error, idle_error } => {
                wait_error == vk::Result::ERROR_DEVICE_LOST
                    || idle_error == vk::Result::ERROR_DEVICE_LOST
            }
        };
        if device_lost {
            Self::DeviceLost(vk::Result::ERROR_DEVICE_LOST)
        } else {
            Self::Command(format!("{error:?}"))
        }
    }
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_shm_texture_device_loss_is_fatal() {
        let error = VulkanSurfaceCompositionError::from_shm_texture_error(
            super::super::super::VulkanShmTextureError::Command(vk::Result::ERROR_DEVICE_LOST),
        );
        assert!(matches!(
            error,
            VulkanSurfaceCompositionError::DeviceLost(vk::Result::ERROR_DEVICE_LOST)
        ));
    }

    #[test]
    fn surface_texture_upload_device_loss_is_fatal() {
        let error = VulkanSurfaceCompositionError::from_surface_texture_cache_error(
            super::super::super::surface_texture_cache::VulkanSurfaceTextureCacheError::Texture(
                super::super::super::VulkanShmTextureError::Command(vk::Result::ERROR_DEVICE_LOST),
            ),
        );
        assert!(matches!(
            error,
            VulkanSurfaceCompositionError::DeviceLost(vk::Result::ERROR_DEVICE_LOST)
        ));
    }

    #[test]
    fn queue_submission_device_loss_is_fatal() {
        let error = VulkanSurfaceCompositionError::from_queue_submit_error(
            VulkanQueueSubmitError::SubmissionUncertain(vk::Result::ERROR_DEVICE_LOST),
        );
        assert!(matches!(
            error,
            VulkanSurfaceCompositionError::DeviceLost(vk::Result::ERROR_DEVICE_LOST)
        ));
    }

    #[test]
    fn device_loss_during_idle_fallback_is_not_hidden() {
        let error = VulkanSurfaceCompositionError::from_queue_submit_error(
            VulkanQueueSubmitError::CompletionUncertain {
                wait_error: vk::Result::TIMEOUT,
                idle_error: vk::Result::ERROR_DEVICE_LOST,
            },
        );
        assert!(matches!(
            error,
            VulkanSurfaceCompositionError::DeviceLost(vk::Result::ERROR_DEVICE_LOST)
        ));
    }

    #[test]
    fn ordinary_completion_failure_remains_a_command_error() {
        let error = VulkanSurfaceCompositionError::from_queue_submit_error(
            VulkanQueueSubmitError::CompletionUncertain {
                wait_error: vk::Result::TIMEOUT,
                idle_error: vk::Result::ERROR_UNKNOWN,
            },
        );
        assert!(matches!(
            error,
            VulkanSurfaceCompositionError::Command(_)
        ));
    }
}
