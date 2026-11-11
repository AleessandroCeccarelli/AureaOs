// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Tracks GPU completion and KMS ownership for reusable frame resources.

use std::{error::Error, fmt};

/// Lifecycle errors returned when a frame resource receives an invalid event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanFrameResourceError {
    AlreadyInUse,
    NotRendering,
    NotSubmitted,
    GpuWorkIncomplete,
    PresentationAlreadyResolved,
    KmsDoesNotOwnResource,
    KmsAlreadyOwnsResource,
}

impl fmt::Display for VulkanFrameResourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::AlreadyInUse => "Frame resource is not available for rendering",
            Self::NotRendering => "Frame resource is not currently being rendered",
            Self::NotSubmitted => "Frame resource has no pending GPU submission",
            Self::GpuWorkIncomplete => "GPU work must complete before KMS can take ownership of the frame",
            Self::PresentationAlreadyResolved => {
                "Frame resource presentation has already been resolved"
            }
            Self::KmsDoesNotOwnResource => "KMS does not currently own this frame resource",
            Self::KmsAlreadyOwnsResource => "KMS already owns this frame resource",
        };

        formatter.write_str(message)
    }
}

impl Error for VulkanFrameResourceError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrameResourceState {
    Available,
    Rendering,
    Submitted {
        gpu_complete: bool,
        presentation_resolved: bool,
        kms_owned: bool,
    },
}

/// Tracks when a GPU-produced image can safely be reused by the renderer.
///
/// GPU completion and KMS release are independent events. A resource becomes
/// available again only after GPU work has completed and presentation has
/// either been skipped or KMS has released its ownership of the buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VulkanFrameResource {
    state: FrameResourceState,
}

impl Default for VulkanFrameResource {
    fn default() -> Self {
        Self::new()
    }
}

impl VulkanFrameResource {
    /// Creates a resource that is immediately available for rendering.
    pub const fn new() -> Self {
        Self {
            state: FrameResourceState::Available,
        }
    }

    /// Returns whether the resource may begin a new rendering operation.
    pub const fn is_available(&self) -> bool {
        matches!(self.state, FrameResourceState::Available)
    }

    /// Begins recording/rendering into this resource.
    pub fn begin_render(&mut self) -> Result<(), VulkanFrameResourceError> {
        if !self.is_available() {
            return Err(VulkanFrameResourceError::AlreadyInUse);
        }

        self.state = FrameResourceState::Rendering;
        Ok(())
    }

    /// Records that rendering commands have been successfully submitted.
    pub fn mark_submitted(&mut self) -> Result<(), VulkanFrameResourceError> {
        if self.state != FrameResourceState::Rendering {
            return Err(VulkanFrameResourceError::NotRendering);
        }

        self.state = FrameResourceState::Submitted {
            gpu_complete: false,
            presentation_resolved: false,
            kms_owned: false,
        };
        Ok(())
    }

    /// Records completion of the GPU submission associated with this resource.
    pub fn mark_gpu_complete(&mut self) -> Result<(), VulkanFrameResourceError> {
        let FrameResourceState::Submitted {
            presentation_resolved,
            kms_owned,
            ..
        } = self.state
        else {
            return Err(VulkanFrameResourceError::NotSubmitted);
        };

        self.update_submitted(true, presentation_resolved, kms_owned);
        Ok(())
    }

    /// Records that an accepted KMS commit now owns the resource.
    ///
    /// KMS may only take ownership after GPU completion. Until explicit
    /// Vulkan-to-KMS fence synchronization is implemented, this conservative
    /// ordering prevents the display engine from reading an unfinished frame.
    pub fn mark_kms_committed(&mut self) -> Result<(), VulkanFrameResourceError> {
        let FrameResourceState::Submitted {
            gpu_complete,
            presentation_resolved,
            kms_owned,
        } = self.state
        else {
            return Err(VulkanFrameResourceError::NotSubmitted);
        };

        if !gpu_complete {
            return Err(VulkanFrameResourceError::GpuWorkIncomplete);
        }
        if presentation_resolved {
            return Err(VulkanFrameResourceError::PresentationAlreadyResolved);
        }
        if kms_owned {
            return Err(VulkanFrameResourceError::KmsAlreadyOwnsResource);
        }

        self.update_submitted(gpu_complete, true, true);
        Ok(())
    }

    /// Records KMS ownership after an exported Vulkan sync-file fence has
    /// been attached to the accepted atomic commit.
    ///
    /// Unlike `mark_kms_committed`, this permits GPU work to remain pending:
    /// the kernel waits on the imported IN_FENCE_FD before scan-out access.
    pub fn mark_kms_committed_with_sync_fd(
        &mut self,
    ) -> Result<(), VulkanFrameResourceError> {
        let FrameResourceState::Submitted {
            gpu_complete,
            presentation_resolved,
            kms_owned,
        } = self.state
        else {
            return Err(VulkanFrameResourceError::NotSubmitted);
        };

        if presentation_resolved {
            return Err(VulkanFrameResourceError::PresentationAlreadyResolved);
        }
        if kms_owned {
            return Err(VulkanFrameResourceError::KmsAlreadyOwnsResource);
        }

        self.update_submitted(gpu_complete, true, true);
        Ok(())
    }

    /// Records that KMS has released a previously committed resource.
    pub fn mark_kms_released(&mut self) -> Result<(), VulkanFrameResourceError> {
        let FrameResourceState::Submitted {
            gpu_complete,
            presentation_resolved,
            kms_owned,
        } = self.state
        else {
            return Err(VulkanFrameResourceError::NotSubmitted);
        };

        if !kms_owned {
            return Err(VulkanFrameResourceError::KmsDoesNotOwnResource);
        }

        self.update_submitted(gpu_complete, presentation_resolved, false);
        Ok(())
    }

    /// Resolves a frame that will not be presented by KMS.
    ///
    /// Call this after an atomic test/commit failure or when presentation is
    /// intentionally skipped. It does not imply that GPU work has completed.
    pub fn mark_presentation_skipped(
        &mut self,
    ) -> Result<(), VulkanFrameResourceError> {
        let FrameResourceState::Submitted {
            gpu_complete,
            presentation_resolved,
            kms_owned,
        } = self.state
        else {
            return Err(VulkanFrameResourceError::NotSubmitted);
        };

        if presentation_resolved {
            return Err(VulkanFrameResourceError::PresentationAlreadyResolved);
        }
        if kms_owned {
            return Err(VulkanFrameResourceError::KmsAlreadyOwnsResource);
        }

        self.update_submitted(gpu_complete, true, false);
        Ok(())
    }

    fn update_submitted(
        &mut self,
        gpu_complete: bool,
        presentation_resolved: bool,
        kms_owned: bool,
    ) {
        self.state = if gpu_complete && presentation_resolved && !kms_owned {
            FrameResourceState::Available
        } else {
            FrameResourceState::Submitted {
                gpu_complete,
                presentation_resolved,
                kms_owned,
            }
        };
    }
}
