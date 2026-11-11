
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
#[cfg(feature = "drm")]
mod drm_image;
mod device_extensions;
mod external_memory;
mod format_capabilities;
mod dynamic_rendering;
mod clear_pass;
mod draw_pass;
mod readback_buffer;
mod offscreen_renderer;
mod graphics_pipeline;
mod image_view;
mod image_transition;
mod rendering_features;
mod shader_module;
mod frame_scheduler;
mod frame_slot;
mod logical_device;
mod memory_type;
mod image_allocator;
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
#[cfg(feature = "drm")]
pub use drm_image::{VulkanDrmImage, VulkanDrmImageAllocator, VulkanDrmImageError, VulkanDrmPlaneLayout};
pub use device_extensions::VulkanDeviceExtensionPlan;
pub use external_memory::VulkanExternalMemoryCapabilities;
pub use format_capabilities::{VulkanFormatCapabilities, VulkanFormatModifierInfo};
pub use dynamic_rendering::{VulkanColorLoad, VulkanDynamicRenderingScope};
pub use clear_pass::{VulkanClearPass, VulkanClearPassError};
pub use draw_pass::{VulkanDrawPass, VulkanDrawPassError};
pub use readback_buffer::{VulkanReadbackBuffer, VulkanReadbackError};
pub use offscreen_renderer::{VulkanOffscreenError, VulkanOffscreenRenderer};
pub use graphics_pipeline::{VulkanGraphicsPipeline, VulkanGraphicsPipelineError, VulkanGraphicsShaders};
pub use image_view::{VulkanImageView, VulkanImageViewError};
pub use image_transition::{VulkanImageState, VulkanImageTransition};
pub use rendering_features::VulkanRenderingFeaturePlan;
pub use shader_module::{VulkanShaderError, VulkanShaderModule};
pub use frame_scheduler::{DEFAULT_FRAMES_IN_FLIGHT, VulkanFrame, VulkanFrameScheduler, VulkanFrameSchedulerError};
pub use logical_device::VulkanLogicalDevice;
pub use memory_type::select_memory_type_index;
pub use image_allocator::{VulkanImage, VulkanImageAllocator, VulkanImageError, VulkanImageRequest};
pub use physical_device::{VulkanDeviceInfo, VulkanDeviceType};
pub use queue::VulkanQueueFamilyInfo;

pub(super) use queue_submitter::VulkanQueueSubmitter;
