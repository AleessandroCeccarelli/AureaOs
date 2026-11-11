// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Tracks GPU completion and external ownership for reusable frame resources.

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
    ForeignOwnershipNotReleased,
    ForeignOwnershipAlreadyReleased,
    ForeignAcquireNotSubmitted,
    ForeignAcquireAlreadySubmitted,
}

impl fmt::Display for VulkanFrameResourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::AlreadyInUse => "Frame resource is not available for rendering",
            Self::NotRendering => "Frame resource is not currently being rendered",
            Self::NotSubmitted => "Frame resource has no pending GPU submission",
            Self::GpuWorkIncomplete => "GPU work must complete before KMS can take ownership of the frame",
            Self::PresentationAlreadyResolved => "Frame resource presentation has already been resolved",
            Self::KmsDoesNotOwnResource => "KMS does not currently own this frame resource",
            Self::KmsAlreadyOwnsResource => "KMS already owns this frame resource",
            Self::ForeignOwnershipNotReleased => "The frame resource has not been released to the foreign queue family",
            Self::ForeignOwnershipAlreadyReleased => "The frame resource is not currently owned by the foreign queue family",
            Self::ForeignAcquireNotSubmitted => "The foreign ownership acquire barrier has not been submitted",
            Self::ForeignAcquireAlreadySubmitted => "The foreign ownership acquire barrier is already pending",
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
        foreign_owned: bool,
        foreign_acquire_pending: bool,
    },
}

/// Tracks when a GPU-produced image can safely be reused.
///
/// GPU completion, KMS release, and completion of the Vulkan acquire barrier
/// are independent conditions. A resource becomes available only after every
/// owner has released it and all Vulkan work has completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VulkanFrameResource {
    state: FrameResourceState,
}

impl Default for VulkanFrameResource {
    fn default() -> Self { Self::new() }
}

impl VulkanFrameResource {
    /// Creates a resource that is immediately available for rendering.
    pub const fn new() -> Self {
        Self { state: FrameResourceState::Available }
    }

    /// Returns whether the resource may begin a new rendering operation.
    pub const fn is_available(&self) -> bool {
        matches!(self.state, FrameResourceState::Available)
    }

    /// Returns whether a GPU submission has been recorded for this resource.
    pub const fn is_submitted(&self) -> bool {
        matches!(self.state, FrameResourceState::Submitted { .. })
    }

    /// Returns whether the resource is submitted and has not been presented or
    /// transferred to KMS yet.
    pub const fn can_present(&self) -> bool {
        matches!(
            self.state,
            FrameResourceState::Submitted {
                presentation_resolved: false,
                kms_owned: false,
                ..
            }
        )
    }

    /// Returns whether the last GPU submission has completed.
    pub const fn is_gpu_complete(&self) -> bool {
        matches!(
            self.state,
            FrameResourceState::Available
                | FrameResourceState::Submitted { gpu_complete: true, .. }
        )
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
            foreign_owned: false,
            foreign_acquire_pending: false,
        };
        Ok(())
    }

    /// Records completion of the GPU submission associated with this resource.
    pub fn mark_gpu_complete(&mut self) -> Result<(), VulkanFrameResourceError> {
        let FrameResourceState::Submitted {
            presentation_resolved, kms_owned, foreign_owned,
            foreign_acquire_pending, ..
        } = self.state else {
            return Err(VulkanFrameResourceError::NotSubmitted);
        };
        self.update_submitted(true, presentation_resolved, kms_owned, foreign_owned, foreign_acquire_pending);
        Ok(())
    }

    /// Records KMS ownership when no explicit IN_FENCE_FD was attached.
    /// The GPU must have completed before this conservative hand-off.
    pub fn mark_kms_committed(&mut self) -> Result<(), VulkanFrameResourceError> {
        let FrameResourceState::Submitted {
            gpu_complete, presentation_resolved, kms_owned, foreign_owned,
            foreign_acquire_pending,
        } = self.state else {
            return Err(VulkanFrameResourceError::NotSubmitted);
        };
        if !gpu_complete { return Err(VulkanFrameResourceError::GpuWorkIncomplete); }
        self.validate_kms_commit(presentation_resolved, kms_owned)?;
        self.update_submitted(gpu_complete, true, true, foreign_owned, foreign_acquire_pending);
        Ok(())
    }

    /// Records KMS ownership after an accepted atomic commit with IN_FENCE_FD.
    pub fn mark_kms_committed_with_sync_fd(&mut self) -> Result<(), VulkanFrameResourceError> {
        let FrameResourceState::Submitted {
            gpu_complete, presentation_resolved, kms_owned, foreign_owned,
            foreign_acquire_pending,
        } = self.state else {
            return Err(VulkanFrameResourceError::NotSubmitted);
        };
        self.validate_kms_commit(presentation_resolved, kms_owned)?;
        self.update_submitted(gpu_complete, true, true, foreign_owned, foreign_acquire_pending);
        Ok(())
    }

    fn validate_kms_commit(
        &self,
        presentation_resolved: bool,
        kms_owned: bool,
    ) -> Result<(), VulkanFrameResourceError> {
        if presentation_resolved { return Err(VulkanFrameResourceError::PresentationAlreadyResolved); }
        if kms_owned { return Err(VulkanFrameResourceError::KmsAlreadyOwnsResource); }
        Ok(())
    }

    /// Records that KMS has released a previously committed resource.
    pub fn mark_kms_released(&mut self) -> Result<(), VulkanFrameResourceError> {
        let FrameResourceState::Submitted {
            gpu_complete, presentation_resolved, kms_owned, foreign_owned,
            foreign_acquire_pending,
        } = self.state else {
            return Err(VulkanFrameResourceError::NotSubmitted);
        };
        if !kms_owned { return Err(VulkanFrameResourceError::KmsDoesNotOwnResource); }
        self.update_submitted(gpu_complete, presentation_resolved, false, foreign_owned, foreign_acquire_pending);
        Ok(())
    }

    /// Resolves a frame that will not be presented by KMS.
    pub fn mark_presentation_skipped(&mut self) -> Result<(), VulkanFrameResourceError> {
        let FrameResourceState::Submitted {
            gpu_complete, presentation_resolved, kms_owned, foreign_owned,
            foreign_acquire_pending,
        } = self.state else {
            return Err(VulkanFrameResourceError::NotSubmitted);
        };
        if presentation_resolved { return Err(VulkanFrameResourceError::PresentationAlreadyResolved); }
        if kms_owned { return Err(VulkanFrameResourceError::KmsAlreadyOwnsResource); }
        self.update_submitted(gpu_complete, true, false, foreign_owned, foreign_acquire_pending);
        Ok(())
    }

    /// Records submission of the Vulkan release barrier to foreign ownership.
    pub fn mark_foreign_release_submitted(&mut self) -> Result<(), VulkanFrameResourceError> {
        let FrameResourceState::Submitted {
            gpu_complete, presentation_resolved, kms_owned, foreign_owned,
            foreign_acquire_pending,
        } = self.state else {
            return Err(VulkanFrameResourceError::NotSubmitted);
        };
        if foreign_owned { return Err(VulkanFrameResourceError::ForeignOwnershipAlreadyReleased); }
        self.update_submitted(gpu_complete, presentation_resolved, kms_owned, true, foreign_acquire_pending);
        Ok(())
    }

    /// Records submission of the Vulkan acquire barrier after KMS has released
    /// the framebuffer. This does not yet make the resource reusable.
    pub fn mark_foreign_acquire_submitted(&mut self) -> Result<(), VulkanFrameResourceError> {
        let FrameResourceState::Submitted {
            gpu_complete, presentation_resolved, kms_owned, foreign_owned,
            foreign_acquire_pending,
        } = self.state else {
            return Err(VulkanFrameResourceError::NotSubmitted);
        };
        if !foreign_owned { return Err(VulkanFrameResourceError::ForeignOwnershipNotReleased); }
        if !presentation_resolved || kms_owned { return Err(VulkanFrameResourceError::KmsAlreadyOwnsResource); }
        if foreign_acquire_pending { return Err(VulkanFrameResourceError::ForeignAcquireAlreadySubmitted); }
        self.update_submitted(gpu_complete, presentation_resolved, kms_owned, foreign_owned, true);
        Ok(())
    }

    /// Records completion of the acquire submission. Call only after its
    /// Vulkan completion fence has signalled.
    pub fn mark_foreign_acquired(&mut self) -> Result<(), VulkanFrameResourceError> {
        let FrameResourceState::Submitted {
            gpu_complete, presentation_resolved, kms_owned, foreign_owned,
            foreign_acquire_pending,
        } = self.state else {
            return Err(VulkanFrameResourceError::NotSubmitted);
        };
        if kms_owned { return Err(VulkanFrameResourceError::KmsAlreadyOwnsResource); }
        if !foreign_owned { return Err(VulkanFrameResourceError::ForeignOwnershipNotReleased); }
        if !foreign_acquire_pending { return Err(VulkanFrameResourceError::ForeignAcquireNotSubmitted); }
        self.update_submitted(gpu_complete, presentation_resolved, kms_owned, false, false);
        Ok(())
    }

    fn update_submitted(
        &mut self,
        gpu_complete: bool,
        presentation_resolved: bool,
        kms_owned: bool,
        foreign_owned: bool,
        foreign_acquire_pending: bool,
    ) {
        self.state = if gpu_complete && presentation_resolved && !kms_owned
            && !foreign_owned && !foreign_acquire_pending {
            FrameResourceState::Available
        } else {
            FrameResourceState::Submitted {
                gpu_complete, presentation_resolved, kms_owned, foreign_owned,
                foreign_acquire_pending,
            }
        };
    }
}
