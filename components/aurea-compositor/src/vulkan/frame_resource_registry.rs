// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Stable identifiers and ownership lookup for Vulkan frames awaiting KMS release.

use std::{collections::BTreeMap, error::Error, fmt};

use super::{VulkanFrameResource, VulkanFrameResourceError};

/// Stable identifier assigned to one registered render-target image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VulkanFrameResourceId(u64);

impl VulkanFrameResourceId {
    /// Numeric identifier suitable for diagnostics and external registries.
    pub const fn get(self) -> u64 { self.0 }
}

/// Errors returned by the frame-resource registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanFrameResourceRegistryError {
    IdentifierExhausted,
    UnknownResource(VulkanFrameResourceId),
    ResourceBusy(VulkanFrameResourceId),
    Lifecycle(VulkanFrameResourceError),
}

impl fmt::Display for VulkanFrameResourceRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IdentifierExhausted => formatter.write_str("Vulkan frame-resource identifiers are exhausted"),
            Self::UnknownResource(id) => write!(formatter, "Unknown Vulkan frame resource {}", id.get()),
            Self::ResourceBusy(id) => write!(formatter, "Vulkan frame resource {} is still owned", id.get()),
            Self::Lifecycle(error) => write!(formatter, "Vulkan frame-resource lifecycle failed: {error}"),
        }
    }
}

impl Error for VulkanFrameResourceRegistryError {}

impl From<VulkanFrameResourceError> for VulkanFrameResourceRegistryError {
    fn from(error: VulkanFrameResourceError) -> Self { Self::Lifecycle(error) }
}

/// Owns lifecycle records for render-target images registered with one output.
///
/// The registry deliberately stores lifecycle state only: Vulkan images and
/// GBM/DRM framebuffer objects remain owned by the output's resource pool.
/// An entry can be removed only after GPU, KMS and foreign-queue ownership
/// have all been resolved.
#[derive(Debug, Default)]
pub struct VulkanFrameResourceRegistry {
    next_id: u64,
    resources: BTreeMap<VulkanFrameResourceId, VulkanFrameResource>,
}

impl VulkanFrameResourceRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self { Self::default() }

    /// Registers a new resource and returns its stable identifier.
    pub fn register(&mut self) -> Result<VulkanFrameResourceId, VulkanFrameResourceRegistryError> {
        let id = VulkanFrameResourceId(self.next_id);
        self.next_id = self.next_id.checked_add(1)
            .ok_or(VulkanFrameResourceRegistryError::IdentifierExhausted)?;
        self.resources.insert(id, VulkanFrameResource::new());
        Ok(id)
    }

    /// Returns the number of registered resources.
    pub fn len(&self) -> usize { self.resources.len() }

    /// Returns whether no resources are registered.
    pub fn is_empty(&self) -> bool { self.resources.is_empty() }

    /// Returns whether a registered resource is ready for rendering.
    pub fn is_available(
        &self,
        id: VulkanFrameResourceId,
    ) -> Result<bool, VulkanFrameResourceRegistryError> {
        Ok(self.resource(id)?.is_available())
    }

    /// Returns whether the resource has completed the render-to-submit transition.
    pub fn is_submitted(
        &self,
        id: VulkanFrameResourceId,
    ) -> Result<bool, VulkanFrameResourceRegistryError> {
        Ok(self.resource(id)?.is_submitted())
    }

    /// Returns whether a submitted resource can be handed to KMS.
    pub fn can_present(
        &self,
        id: VulkanFrameResourceId,
    ) -> Result<bool, VulkanFrameResourceRegistryError> {
        Ok(self.resource(id)?.can_present())
    }

    /// Returns whether the resource's GPU submission has completed.
    pub fn is_gpu_complete(
        &self,
        id: VulkanFrameResourceId,
    ) -> Result<bool, VulkanFrameResourceRegistryError> {
        Ok(self.resource(id)?.is_gpu_complete())
    }

    /// Begins rendering into a registered resource.
    pub fn begin_render(
        &mut self,
        id: VulkanFrameResourceId,
    ) -> Result<(), VulkanFrameResourceRegistryError> {
        self.resource_mut(id)?.begin_render()?;
        Ok(())
    }

    /// Records successful GPU queue submission.
    pub fn mark_submitted(
        &mut self,
        id: VulkanFrameResourceId,
    ) -> Result<(), VulkanFrameResourceRegistryError> {
        self.resource_mut(id)?.mark_submitted()?;
        Ok(())
    }

    /// Records completion of the GPU submission.
    pub fn mark_gpu_complete(
        &mut self,
        id: VulkanFrameResourceId,
    ) -> Result<(), VulkanFrameResourceRegistryError> {
        self.resource_mut(id)?.mark_gpu_complete()?;
        Ok(())
    }

    /// Records KMS ownership after a conservative GPU-complete hand-off.
    pub fn mark_kms_committed(
        &mut self,
        id: VulkanFrameResourceId,
    ) -> Result<(), VulkanFrameResourceRegistryError> {
        self.resource_mut(id)?.mark_kms_committed()?;
        Ok(())
    }

    /// Records KMS ownership after a sync-file-fenced atomic commit.
    pub fn mark_kms_committed_with_sync_fd(
        &mut self,
        id: VulkanFrameResourceId,
    ) -> Result<(), VulkanFrameResourceRegistryError> {
        self.resource_mut(id)?.mark_kms_committed_with_sync_fd()?;
        Ok(())
    }

    /// Records the page-flip release of a previously scanned-out resource.
    pub fn mark_kms_released(
        &mut self,
        id: VulkanFrameResourceId,
    ) -> Result<(), VulkanFrameResourceRegistryError> {
        self.resource_mut(id)?.mark_kms_released()?;
        Ok(())
    }

    /// Resolves a submitted frame that will not be presented.
    pub fn mark_presentation_skipped(
        &mut self,
        id: VulkanFrameResourceId,
    ) -> Result<(), VulkanFrameResourceRegistryError> {
        self.resource_mut(id)?.mark_presentation_skipped()?;
        Ok(())
    }

    /// Records that a foreign-queue release barrier was submitted.
    pub fn mark_foreign_release_submitted(
        &mut self,
        id: VulkanFrameResourceId,
    ) -> Result<(), VulkanFrameResourceRegistryError> {
        self.resource_mut(id)?.mark_foreign_release_submitted()?;
        Ok(())
    }

    /// Records submission of the acquire barrier after KMS released the image.
    pub fn mark_foreign_acquire_submitted(
        &mut self,
        id: VulkanFrameResourceId,
    ) -> Result<(), VulkanFrameResourceRegistryError> {
        self.resource_mut(id)?.mark_foreign_acquire_submitted()?;
        Ok(())
    }

    /// Records completion of the foreign-queue acquire submission.
    pub fn mark_foreign_acquired(
        &mut self,
        id: VulkanFrameResourceId,
    ) -> Result<(), VulkanFrameResourceRegistryError> {
        self.resource_mut(id)?.mark_foreign_acquired()?;
        Ok(())
    }

    /// Removes an entry only when every owner has released the resource.
    pub fn unregister(
        &mut self,
        id: VulkanFrameResourceId,
    ) -> Result<(), VulkanFrameResourceRegistryError> {
        if !self.resource(id)?.is_available() {
            return Err(VulkanFrameResourceRegistryError::ResourceBusy(id));
        }
        self.resources.remove(&id);
        Ok(())
    }

    fn resource(
        &self,
        id: VulkanFrameResourceId,
    ) -> Result<&VulkanFrameResource, VulkanFrameResourceRegistryError> {
        self.resources.get(&id)
            .ok_or(VulkanFrameResourceRegistryError::UnknownResource(id))
    }

    fn resource_mut(
        &mut self,
        id: VulkanFrameResourceId,
    ) -> Result<&mut VulkanFrameResource, VulkanFrameResourceRegistryError> {
        self.resources.get_mut(&id)
            .ok_or(VulkanFrameResourceRegistryError::UnknownResource(id))
    }
}
