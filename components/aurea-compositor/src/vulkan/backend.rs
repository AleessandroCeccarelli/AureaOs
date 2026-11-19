// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan rendering-backend initialization and renderer integration.

#[cfg(feature = "drm")]
use std::sync::Mutex;
use std::{error::Error, fmt, sync::Arc};

use smithay::{backend::vulkan::PhysicalDevice, reexports::ash::vk};

#[cfg(feature = "drm")]
use smithay::backend::drm::DrmNode;

use crate::renderer::RenderPreference;

use super::{
    VulkanCommandBuffer, VulkanContext, VulkanDeviceExtensionPlan, VulkanDeviceInfo,
    VulkanInitError, VulkanLogicalDevice,
};

/// Error returned while recording Vulkan commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VulkanCommandError(pub vk::Result);

impl fmt::Display for VulkanCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Vulkan command-buffer error: {:?}", self.0)
    }
}
impl Error for VulkanCommandError {}

/// Owns the Vulkan context, selected GPU and logical device.
pub struct VulkanBackend {
    #[cfg(feature = "drm")]
    display_color: Mutex<super::VulkanDisplayColorManager>,
    #[cfg(feature = "drm")]
    surface_pipeline: Mutex<super::VulkanSurfacePipelineCache>,
    context: Arc<VulkanContext>,
    selected_device_info: VulkanDeviceInfo,
    physical_device_index: usize,
    device: Arc<VulkanLogicalDevice>,
}

impl VulkanBackend {
    /// Initializes Vulkan using the requested GPU preference.
    ///
    /// This constructor is for generic/offscreen work. DRM presentation should
    /// use new_for_drm_node so the selected GPU is bound to the output device.
    pub fn new(preference: RenderPreference) -> Result<Self, VulkanInitError> {
        let context = Arc::new(VulkanContext::new()?);
        let physical_device = context
            .select_device(preference)
            .ok_or_else(|| VulkanInitError::new("No graphics-capable Vulkan device found"))?;
        let physical_device_index = context
            .physical_devices()
            .iter()
            .position(|candidate| std::ptr::eq(candidate, physical_device))
            .ok_or_else(|| {
                VulkanInitError::new("Selected Vulkan device is not owned by its context")
            })?;
        Self::from_selected_device(context, physical_device_index, false)
    }

    /// Initializes Vulkan for the physical device associated with a DRM node.
    #[cfg(feature = "drm")]
    pub fn new_for_drm_node(
        node: DrmNode,
        preference: RenderPreference,
    ) -> Result<Self, VulkanInitError> {
        let context = Arc::new(VulkanContext::new()?);
        let physical_device = context
            .select_device_for_drm_node(node, preference)
            .ok_or_else(|| {
                VulkanInitError::new(
                    "No graphics-capable Vulkan device matches the requested DRM node",
                )
            })?;
        let physical_device_index = context
            .physical_devices()
            .iter()
            .position(|candidate| std::ptr::eq(candidate, physical_device))
            .ok_or_else(|| {
                VulkanInitError::new("Selected Vulkan device is not owned by its context")
            })?;
        Self::from_selected_device(context, physical_device_index, true)
    }

    fn from_selected_device(
        context: Arc<VulkanContext>,
        physical_device_index: usize,
        require_dmabuf_interop: bool,
    ) -> Result<Self, VulkanInitError> {
        let physical_device = context
            .physical_devices()
            .get(physical_device_index)
            .ok_or_else(|| VulkanInitError::new("Selected Vulkan device index is invalid"))?;
        let queue_family_index = context
            .graphics_queue_family(physical_device)
            .ok_or_else(|| VulkanInitError::new("Selected device has no graphics queue"))?;
        let extension_plan = VulkanDeviceExtensionPlan::from_device(physical_device);

        if require_dmabuf_interop && !extension_plan.supports_dmabuf_interop() {
            return Err(VulkanInitError::new(format!(
                "Selected Vulkan device lacks required DMA-BUF extensions: {}",
                extension_plan.missing_extensions().join(", ")
            )));
        }

        let selected_device_info = Self::build_selected_device_info(&context, physical_device);
        let device = Arc::new(VulkanLogicalDevice::new(
            physical_device,
            queue_family_index,
            extension_plan,
            Arc::clone(&context),
        )?);

        #[cfg(feature = "drm")]
        let surface_pipeline = super::VulkanSurfacePipelineCache::new(Arc::clone(&device))
            .map_err(|error| VulkanInitError::new(error.to_string()))?;

        Ok(Self {
            #[cfg(feature = "drm")]
            display_color: Mutex::new(super::VulkanDisplayColorManager::new(Arc::clone(&device))),
            #[cfg(feature = "drm")]
            surface_pipeline: Mutex::new(surface_pipeline),
            context,
            selected_device_info,
            physical_device_index,
            device,
        })
    }

    /// Returns the Vulkan context.
    pub fn context(&self) -> &VulkanContext {
        self.context.as_ref()
    }

    /// Returns information about the selected physical GPU.
    pub fn selected_device_info(&self) -> &VulkanDeviceInfo {
        &self.selected_device_info
    }

    /// Returns the exact physical device used to create this logical device.
    pub fn selected_physical_device(&self) -> &PhysicalDevice {
        &self.context.physical_devices()[self.physical_device_index]
    }

    /// Returns the logical device.
    pub fn device(&self) -> &VulkanLogicalDevice {
        self.device.as_ref()
    }

    /// Returns shared ownership for resources that must outlive this backend borrow.
    #[cfg(feature = "drm")]
    pub fn shared_device(&self) -> Arc<VulkanLogicalDevice> {
        Arc::clone(&self.device)
    }

    /// Returns the per-device display-color pipeline cache.
    #[cfg(feature = "drm")]
    pub(crate) fn display_color_manager(&self) -> &Mutex<super::VulkanDisplayColorManager> {
        &self.display_color
    }

    /// Returns the per-device SHM surface pipeline and resource cache.
    #[cfg(feature = "drm")]
    pub(crate) fn surface_pipeline_cache(&self) -> &Mutex<super::VulkanSurfacePipelineCache> {
        &self.surface_pipeline
    }

    /// Records an empty one-time command buffer.
    ///
    /// This exercises command-buffer allocation, recording and finalization.
    /// It does not submit work to the queue or present a frame.
    pub fn record_empty_command_buffer(
        &self,
    ) -> Result<VulkanCommandBuffer<'_>, VulkanCommandError> {
        let mut command_buffer =
            VulkanCommandBuffer::new(&self.device).map_err(VulkanCommandError)?;
        command_buffer
            .begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .map_err(VulkanCommandError)?;
        command_buffer.end().map_err(VulkanCommandError)?;
        Ok(command_buffer)
    }

    fn build_selected_device_info(
        context: &VulkanContext,
        device: &PhysicalDevice,
    ) -> VulkanDeviceInfo {
        let has_graphics_queue = context
            .queue_families(device)
            .iter()
            .any(|family| family.supports_graphics && family.queue_count > 0);
        VulkanDeviceInfo::from_device(device, has_graphics_queue)
    }
}
