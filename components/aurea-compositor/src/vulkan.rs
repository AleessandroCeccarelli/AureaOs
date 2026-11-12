// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan integration for the AureaOS compositor.

mod backend;
#[cfg(feature = "drm")]
mod buffer_allocator;
mod clear_pass;
mod command_buffer;
mod context;
mod device_extensions;
#[cfg(feature = "drm")]
mod dmabuf_layout;
mod draw_pass;
#[cfg(feature = "drm")]
mod drm_compatibility;
#[cfg(feature = "drm")]
mod drm_framebuffer;
#[cfg(feature = "drm")]
mod drm_image;
#[cfg(feature = "drm")]
mod drm_frame_pool;
mod dynamic_rendering;
mod external_memory;
mod format_capabilities;
#[cfg(feature = "drm")]
mod frame_resource;
#[cfg(feature = "drm")]
mod frame_resource_registry;
mod frame_scheduler;
mod frame_slot;
mod graphics_pipeline;
mod image_allocator;
mod image_transition;
mod image_view;
mod logical_device;
mod memory_type;
mod offscreen_renderer;
mod physical_device;
mod queue;
mod queue_submitter;
mod readback_buffer;
mod rendering_features;
mod shader_module;

use std::{error::Error, fmt};

/// Error returned while initializing Vulkan.
#[derive(Debug)]
pub struct VulkanInitError {
    message: String,
}

impl VulkanInitError {
    pub(super) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
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
pub use clear_pass::{VulkanClearPass, VulkanClearPassError};
pub use command_buffer::VulkanCommandBuffer;
pub use context::VulkanContext;
pub use device_extensions::VulkanDeviceExtensionPlan;
#[cfg(feature = "drm")]
pub use dmabuf_layout::{DmabufImportLayout, DmabufLayoutError, DmabufPlaneLayout};
pub use draw_pass::{VulkanDrawPass, VulkanDrawPassError};
#[cfg(feature = "drm")]
pub use drm_compatibility::{
    DrmVulkanCompatibility, DrmVulkanCompatibilityStatus, DrmVulkanFormat,
};
#[cfg(feature = "drm")]
pub use drm_framebuffer::{VulkanDrmFramebuffer, VulkanDrmFramebufferError};
#[cfg(feature = "drm")]
pub use drm_image::{
    VulkanDrmImage, VulkanDrmImageAllocator, VulkanDrmImageError, VulkanDrmPlaneLayout,
};
#[cfg(feature = "drm")]
pub use drm_frame_pool::{
    VulkanDrmFramePool, VulkanDrmFramePoolError, VulkanDrmScanoutFrame,
};
pub use dynamic_rendering::{VulkanColorLoad, VulkanDynamicRenderingScope};
pub use external_memory::VulkanExternalMemoryCapabilities;
pub use format_capabilities::{VulkanFormatCapabilities, VulkanFormatModifierInfo};
#[cfg(feature = "drm")]
pub use frame_resource::{VulkanFrameResource, VulkanFrameResourceError};
#[cfg(feature = "drm")]
pub use frame_resource_registry::{
    VulkanFrameResourceId, VulkanFrameResourceRegistry, VulkanFrameResourceRegistryError,
};
pub use frame_scheduler::{
    DEFAULT_FRAMES_IN_FLIGHT, VulkanFrame, VulkanFrameScheduler, VulkanFrameSchedulerError,
};
pub use graphics_pipeline::{
    VulkanGraphicsPipeline, VulkanGraphicsPipelineError, VulkanGraphicsShaders,
};
pub use image_allocator::{
    VulkanImage, VulkanImageAllocator, VulkanImageError, VulkanImageRequest,
};
pub use image_transition::{VulkanImageState, VulkanImageTransition, VulkanImageTransitionError};
pub use image_view::{VulkanImageView, VulkanImageViewError};
pub use logical_device::VulkanLogicalDevice;
pub use memory_type::select_memory_type_index;
pub use offscreen_renderer::{VulkanOffscreenError, VulkanOffscreenRenderer};
pub use physical_device::{VulkanDeviceInfo, VulkanDeviceType};
pub use queue::VulkanQueueFamilyInfo;
pub use readback_buffer::{VulkanReadbackBuffer, VulkanReadbackError};
pub use rendering_features::VulkanRenderingFeaturePlan;
pub use shader_module::{VulkanShaderError, VulkanShaderModule};

pub(super) use queue_submitter::VulkanQueueSubmitter;
