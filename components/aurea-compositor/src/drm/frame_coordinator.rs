// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Coordinates Vulkan image ownership with one output's KMS page-flip lifecycle.

use std::{collections::HashMap, error::Error, fmt};

use crate::vulkan::{
    VulkanFrameResourceId,
    VulkanFrameResourceRegistry,
    VulkanFrameResourceRegistryError,
};

use super::presentation::{
    DrmFramebufferId,
    DrmPresentationError,
    DrmPresentationLifecycle,
};

/// Errors returned while coordinating one output's Vulkan/KMS frame lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrmOutputFrameError {
    DuplicateFramebuffer(DrmFramebufferId),
    UnknownFramebuffer(DrmFramebufferId),
    ResourceNotSubmitted(DrmFramebufferId),
    FramebufferStillScannedOut(DrmFramebufferId),
    Lifecycle(DrmPresentationError),
    Resource(VulkanFrameResourceRegistryError),
}

impl fmt::Display for DrmOutputFrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateFramebuffer(id) => write!(formatter, "DRM framebuffer {} is already registered", id.value()),
            Self::UnknownFramebuffer(id) => write!(formatter, "DRM framebuffer {} is not registered", id.value()),
            Self::ResourceNotSubmitted(id) => write!(formatter, "DRM framebuffer {} has not been submitted to the GPU", id.value()),
            Self::FramebufferStillScannedOut(id) => write!(formatter, "DRM framebuffer {} is still pending or scanned out", id.value()),
            Self::Lifecycle(error) => write!(formatter, "KMS presentation lifecycle failed: {error}"),
            Self::Resource(error) => write!(formatter, "Vulkan frame-resource lifecycle failed: {error}"),
        }
    }
}

impl Error for DrmOutputFrameError {}

impl From<DrmPresentationError> for DrmOutputFrameError {
    fn from(error: DrmPresentationError) -> Self { Self::Lifecycle(error) }
}

impl From<VulkanFrameResourceRegistryError> for DrmOutputFrameError {
    fn from(error: VulkanFrameResourceRegistryError) -> Self { Self::Resource(error) }
}

/// Coordinates the framebuffer pool and KMS state for exactly one output.
///
/// Framebuffer objects themselves remain owned by the output's framebuffer
/// pool. This coordinator owns only their stable identities and the lifecycle
/// records needed to prevent reuse before GPU and KMS release.
#[derive(Debug, Default)]
pub struct DrmOutputFrameCoordinator {
    resources: VulkanFrameResourceRegistry,
    framebuffer_resources: HashMap<DrmFramebufferId, VulkanFrameResourceId>,
    presentation: DrmPresentationLifecycle,
}

impl DrmOutputFrameCoordinator {
    /// Creates an empty coordinator for one output/CRTC.
    pub fn new() -> Self { Self::default() }

    /// Registers a framebuffer and its associated Vulkan lifecycle record.
    pub fn register_framebuffer(
        &mut self,
        framebuffer: DrmFramebufferId,
    ) -> Result<VulkanFrameResourceId, DrmOutputFrameError> {
        if self.framebuffer_resources.contains_key(&framebuffer) {
            return Err(DrmOutputFrameError::DuplicateFramebuffer(framebuffer));
        }
        let resource = self.resources.register()?;
        self.framebuffer_resources.insert(framebuffer, resource);
        Ok(resource)
    }

    /// Returns the Vulkan resource associated with a registered framebuffer.
    pub fn resource_for(
        &self,
        framebuffer: DrmFramebufferId,
    ) -> Result<VulkanFrameResourceId, DrmOutputFrameError> {
        self.framebuffer_resources.get(&framebuffer).copied()
            .ok_or(DrmOutputFrameError::UnknownFramebuffer(framebuffer))
    }

    /// Begins rendering into a registered framebuffer's Vulkan image.
    pub fn begin_render(
        &mut self,
        framebuffer: DrmFramebufferId,
    ) -> Result<(), DrmOutputFrameError> {
        let resource = self.resource_for(framebuffer)?;
        self.resources.begin_render(resource)?;
        Ok(())
    }

    /// Records successful GPU queue submission for a rendered framebuffer.
    pub fn mark_submitted(
        &mut self,
        framebuffer: DrmFramebufferId,
    ) -> Result<(), DrmOutputFrameError> {
        let resource = self.resource_for(framebuffer)?;
        self.resources.mark_submitted(resource)?;
        Ok(())
    }

    /// Records GPU completion independently of KMS page-flip completion.
    pub fn mark_gpu_complete(
        &mut self,
        framebuffer: DrmFramebufferId,
    ) -> Result<(), DrmOutputFrameError> {
        let resource = self.resource_for(framebuffer)?;
        self.resources.mark_gpu_complete(resource)?;
        Ok(())
    }

    /// Begins the atomic TEST_ONLY transaction for a submitted framebuffer.
    pub fn begin_presentation(
        &mut self,
        framebuffer: DrmFramebufferId,
    ) -> Result<(), DrmOutputFrameError> {
        let resource = self.resource_for(framebuffer)?;
        if !self.resources.is_submitted(resource)? {
            return Err(DrmOutputFrameError::ResourceNotSubmitted(framebuffer));
        }
        self.presentation.begin_test(framebuffer)?;
        Ok(())
    }

    /// Records successful atomic TEST_ONLY validation.
    pub fn presentation_test_passed(&mut self) -> Result<(), DrmOutputFrameError> {
        self.presentation.test_passed()?;
        Ok(())
    }

    /// Resolves a failed TEST_ONLY attempt without releasing unfinished GPU work.
    pub fn presentation_test_failed(
        &mut self,
    ) -> Result<DrmFramebufferId, DrmOutputFrameError> {
        let framebuffer = self.presentation.test_failed()?;
        let resource = self.resource_for(framebuffer)?;
        self.resources.mark_presentation_skipped(resource)?;
        Ok(framebuffer)
    }

    /// Records an accepted KMS commit and transfers ownership to KMS.
    ///
    /// Use the sync-file variant only when the exact exported fence was
    /// attached to every active plane in the accepted atomic commit.
    pub fn presentation_commit_accepted(
        &mut self,
        sync_file_attached: bool,
    ) -> Result<(), DrmOutputFrameError> {
        self.presentation.commit_accepted()?;
        let framebuffer = self.presentation.pending_candidate()
            .ok_or(DrmOutputFrameError::Lifecycle(DrmPresentationError::NoTransaction))?;
        let resource = self.resource_for(framebuffer)?;
        if sync_file_attached {
            self.resources.mark_kms_committed_with_sync_fd(resource)?;
        } else {
            self.resources.mark_kms_committed(resource)?;
        }
        Ok(())
    }

    /// Resolves a failed KMS commit and leaves the previous scan-out untouched.
    pub fn presentation_commit_failed(
        &mut self,
    ) -> Result<DrmFramebufferId, DrmOutputFrameError> {
        let framebuffer = self.presentation.commit_failed()?;
        let resource = self.resource_for(framebuffer)?;
        self.resources.mark_presentation_skipped(resource)?;
        Ok(framebuffer)
    }

    /// Completes a page flip and releases the previous framebuffer's KMS ownership.
    ///
    /// The returned identity is the framebuffer no longer scanned out. It is
    /// reusable only if its GPU and foreign-queue work have also completed.
    pub fn page_flip_complete(
        &mut self,
        displayed: DrmFramebufferId,
    ) -> Result<Option<DrmFramebufferId>, DrmOutputFrameError> {
        let released = self.presentation.page_flip_complete(displayed)?;
        if let Some(previous) = released {
            let resource = self.resource_for(previous)?;
            self.resources.mark_kms_released(resource)?;
        }
        Ok(released)
    }

    /// Returns the framebuffer currently scanned out by this output.
    pub fn current(&self) -> Option<DrmFramebufferId> {
        self.presentation.current()
    }

    /// Returns the framebuffer awaiting its page-flip event.
    pub fn pending(&self) -> Option<DrmFramebufferId> {
        self.presentation.pending_candidate()
    }

    /// Returns whether a framebuffer's Vulkan image is ready for reuse.
    pub fn is_available(
        &self,
        framebuffer: DrmFramebufferId,
    ) -> Result<bool, DrmOutputFrameError> {
        let resource = self.resource_for(framebuffer)?;
        Ok(self.resources.is_available(resource)?)
    }

    /// Unregisters a framebuffer only after it is no longer scanned out or pending.
    pub fn unregister_framebuffer(
        &mut self,
        framebuffer: DrmFramebufferId,
    ) -> Result<(), DrmOutputFrameError> {
        if self.current() == Some(framebuffer) || self.pending() == Some(framebuffer) {
            return Err(DrmOutputFrameError::FramebufferStillScannedOut(framebuffer));
        }
        let resource = self.resource_for(framebuffer)?;
        self.resources.unregister(resource)?;
        self.framebuffer_resources.remove(&framebuffer);
        Ok(())
    }

    /// Number of framebuffer resources registered for this output.
    pub fn len(&self) -> usize { self.framebuffer_resources.len() }

    /// Whether this output has no registered framebuffer resources.
    pub fn is_empty(&self) -> bool { self.framebuffer_resources.is_empty() }
}
