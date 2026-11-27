// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan integration for the AureaOS compositor.

mod backend;
#[cfg(feature = "drm")]
mod buffer_allocator;
mod capture_target;
mod capture_target_pool;
mod clear_pass;
mod command_buffer;
mod context;
mod desktop_icon_texture_cache;
mod device_extensions;
mod device_owner;
#[cfg(feature = "drm")]
mod display_color_manager;
#[cfg(feature = "drm")]
mod display_color_pass;
#[cfg(feature = "drm")]
mod dmabuf_layout;
mod draw_pass;
#[cfg(feature = "drm")]
mod drm_compatibility;
#[cfg(feature = "drm")]
mod drm_frame_pool;
#[cfg(feature = "drm")]
mod drm_framebuffer;
#[cfg(feature = "drm")]
mod drm_image;
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
mod legacy_render_pass;
mod logical_device;
mod logical_device_factory;
mod memory_type;
mod native_cursor_texture;
mod offscreen_renderer;
mod pending_upload;
mod physical_device;
mod queue;
mod queue_submitter;
mod readback_buffer;
mod readback_buffer_device;
mod readback_buffer_lease;
mod readback_buffer_pool;
mod readback_coordinator;
mod readback_error;
mod readback_submission;
mod rendering_features;
mod sampler;
mod shader_module;
mod shared_image;
mod shm_texture;
#[cfg(feature = "drm")]
pub(crate) mod surface_compositor;
mod surface_pipeline_cache;
mod surface_texture_cache;
mod texture_binding;
mod texture_layout;
mod upload_batch;
mod upload_buffer;
mod wallpaper_texture;
mod wallpaper_texture_cache;
mod xcursor_file;

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
pub use capture_target::{VULKAN_CAPTURE_FORMAT, VulkanCaptureTarget, VulkanCaptureTargetError};
pub use capture_target_pool::{
    VulkanCaptureTargetLease, VulkanCaptureTargetPool, VulkanCaptureTargetPoolStats,
};
pub use clear_pass::{VulkanClearPass, VulkanClearPassError};
pub use command_buffer::VulkanCommandBuffer;
pub use context::VulkanContext;
pub use device_extensions::VulkanDeviceExtensionPlan;
#[cfg(feature = "drm")]
pub(crate) use display_color_manager::VulkanDisplayColorManager;
#[cfg(feature = "drm")]
pub use display_color_pass::{VulkanDisplayColorPass, VulkanDisplayColorPassError};
#[cfg(feature = "drm")]
pub use dmabuf_layout::{DmabufImportLayout, DmabufLayoutError, DmabufPlaneLayout};
pub use draw_pass::{VulkanDrawPass, VulkanDrawPassError};
#[cfg(feature = "drm")]
pub use drm_compatibility::{
    DrmVulkanCompatibility, DrmVulkanCompatibilityStatus, DrmVulkanFormat,
};
#[cfg(feature = "drm")]
pub use drm_frame_pool::{VulkanDrmFramePool, VulkanDrmFramePoolError, VulkanDrmScanoutFrame};
#[cfg(feature = "drm")]
pub use drm_framebuffer::{VulkanDrmFramebuffer, VulkanDrmFramebufferError};
#[cfg(feature = "drm")]
pub use drm_image::{
    VulkanDrmImage, VulkanDrmImageAllocator, VulkanDrmImageError, VulkanDrmPlaneLayout,
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
pub use legacy_render_pass::{
    VulkanLegacyRenderPassError, VulkanLegacyRenderPassScope, VulkanLegacyRenderTarget,
};
pub use logical_device::VulkanLogicalDevice;
pub use memory_type::select_memory_type_index;
pub(crate) use native_cursor_texture::{VulkanNativeCursorTexture, VulkanNativeCursorTextureError};
pub use offscreen_renderer::{VulkanOffscreenError, VulkanOffscreenRenderer};
pub use physical_device::{VulkanDeviceInfo, VulkanDeviceType};
pub use queue::VulkanQueueFamilyInfo;
pub use readback_buffer::VulkanReadbackBuffer;
pub use readback_buffer_lease::VulkanReadbackBufferLease;
pub use readback_buffer_pool::{VulkanReadbackBufferPool, VulkanReadbackPoolStats};
pub use readback_coordinator::{VulkanReadbackCoordinator, VulkanReadbackPoll};
pub use readback_error::VulkanReadbackError;
pub use readback_submission::VulkanReadbackSubmission;
pub use rendering_features::VulkanRenderingFeaturePlan;
pub use sampler::{VulkanSampler, VulkanSamplerError};
pub use shader_module::{VulkanShaderError, VulkanShaderModule};
pub use shared_image::{VulkanSharedImage, VulkanSharedImageError};
pub use shm_texture::{VulkanShmTexture, VulkanShmTextureError};
pub(crate) use surface_pipeline_cache::VulkanSurfacePipelineCache;
pub use texture_binding::{
    VulkanTextureBinding, VulkanTextureBindingError, VulkanTextureBindingPool,
};
pub use texture_layout::{VulkanTextureDescriptorSetLayout, VulkanTextureLayoutError};
pub use upload_buffer::{VulkanUploadBuffer, VulkanUploadError};
pub use wallpaper_texture::{VulkanWallpaperTexture, VulkanWallpaperTextureError};

pub(super) use device_owner::VulkanDeviceOwner;
pub(super) use pending_upload::VulkanPendingUpload;
pub(super) use queue_submitter::VulkanQueueSubmitter;
pub(super) use upload_batch::VulkanUploadBatch;
