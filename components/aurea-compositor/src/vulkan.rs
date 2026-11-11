
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan integration for the AureaOS compositor.

mod backend;
#[cfg(feature = "drm")]
mod buffer_allocator;
mod command_buffer;
mod context;
#[cfg(feature = "drm")]
mod dmabuf_layout;
#[cfg(feature = "drm")]
mod drm_compatibility;
mod device_extensions;
mod external_memory;
mod format_capabilities;
mod logical_device;
mod physical_device;
mod queue;
mod queue_submitter;

use std::{error::Error, fmt};

/// Error returned while initializing Vulkan.
#[derive(Debug)]
pub struct VulkanInitError {
    message: String,
}

impl VulkanInitError {
    pub(super) fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl fmt::Display for VulkanInitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}
impl Error for VulkanInitError {}

pub use backend::VulkanBackend;
#[cfg(feature = "drm")]
pub use buffer_allocator::{VulkanBufferAllocator, VulkanBufferError, VulkanRenderTargetRequest};
pub use command_buffer::VulkanCommandBuffer;
pub use context::VulkanContext;
#[cfg(feature = "drm")]
pub use dmabuf_layout::{DmabufImportLayout, DmabufLayoutError, DmabufPlaneLayout};
#[cfg(feature = "drm")]
pub use drm_compatibility::{
    DrmVulkanCompatibility,
    DrmVulkanCompatibilityStatus,
    DrmVulkanFormat,
};
pub use device_extensions::VulkanDeviceExtensionPlan;
pub use external_memory::VulkanExternalMemoryCapabilities;
pub use format_capabilities::{VulkanFormatCapabilities, VulkanFormatModifierInfo};
pub use logical_device::VulkanLogicalDevice;
pub use physical_device::{VulkanDeviceInfo, VulkanDeviceType};
pub use queue::VulkanQueueFamilyInfo;

pub(super) use queue_submitter::VulkanQueueSubmitter;
