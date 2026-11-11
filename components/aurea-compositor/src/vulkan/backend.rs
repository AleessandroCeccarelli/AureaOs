// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan rendering-backend initialization and renderer integration.

use std::{error::Error, fmt};

use smithay::{backend::vulkan::PhysicalDevice, reexports::ash::vk};

#[cfg(feature = "drm")]
use smithay::backend::drm::DrmNode;

use crate::{
    render::RenderScene,
    renderer::{RenderBackend, RenderPreference, RenderTarget},
};

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

/// Error returned when Vulkan cannot render or present a frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VulkanRenderError {
    /// Swapchain and output presentation are not implemented yet.
    PresentationUnavailable,
}
impl fmt::Display for VulkanRenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PresentationUnavailable => formatter.write_str(
                "Vulkan output presentation is not implemented yet"
            ),
        }
    }
}
impl Error for VulkanRenderError {}

/// Owns the Vulkan context, selected GPU and logical device.
pub struct VulkanBackend {
    context: VulkanContext,
    selected_device_info: VulkanDeviceInfo,
    device: VulkanLogicalDevice,
}

impl VulkanBackend {
    /// Initializes Vulkan using the requested GPU preference.
    ///
    /// This constructor is for generic/offscreen work. DRM presentation should
    /// use new_for_drm_node so the selected GPU is bound to the output device.
    pub fn new(preference: RenderPreference) -> Result<Self, VulkanInitError> {
        let context = VulkanContext::new()?;
        let physical_device = context.select_device(preference)
            .ok_or_else(|| VulkanInitError::new("No graphics-capable Vulkan device found"))?;
        let physical_device_index = context.physical_devices()
            .iter()
            .position(|candidate| std::ptr::eq(candidate, physical_device))
            .ok_or_else(|| VulkanInitError::new("Selected Vulkan device is not owned by its context"))?;
        Self::from_selected_device(context, physical_device_index, false)
    }

    /// Initializes Vulkan for the physical device associated with a DRM node.
    #[cfg(feature = "drm")]
    pub fn new_for_drm_node(
        node: DrmNode,
        preference: RenderPreference,
    ) -> Result<Self, VulkanInitError> {
        let context = VulkanContext::new()?;
        let physical_device = context.select_device_for_drm_node(node, preference)
            .ok_or_else(|| VulkanInitError::new(
                "No graphics-capable Vulkan device matches the requested DRM node"
            ))?;
        let physical_device_index = context.physical_devices()
            .iter()
            .position(|candidate| std::ptr::eq(candidate, physical_device))
            .ok_or_else(|| VulkanInitError::new("Selected Vulkan device is not owned by its context"))?;
        Self::from_selected_device(context, physical_device_index, true)
    }

    fn from_selected_device(
        context: VulkanContext,
        physical_device_index: usize,
        require_dmabuf_interop: bool,
    ) -> Result<Self, VulkanInitError> {
        let physical_device = context.physical_devices()
            .get(physical_device_index)
            .ok_or_else(|| VulkanInitError::new("Selected Vulkan device index is invalid"))?;
        let queue_family_index = context.graphics_queue_family(physical_device)
            .ok_or_else(|| VulkanInitError::new("Selected device has no graphics queue"))?;
        let extension_plan = VulkanDeviceExtensionPlan::from_device(physical_device);

        if require_dmabuf_interop && !extension_plan.supports_dmabuf_interop() {
            return Err(VulkanInitError::new(format!(
                "Selected Vulkan device lacks required DMA-BUF extensions: {}",
                extension_plan.missing_extensions().join(", ")
            )));
        }

        let selected_device_info = Self::build_selected_device_info(&context, physical_device);
        let device = VulkanLogicalDevice::new(
            physical_device,
            queue_family_index,
            extension_plan,
        )?;

        Ok(Self { context, selected_device_info, device })
    }

    /// Returns the Vulkan context.
    pub fn context(&self) -> &VulkanContext { &self.context }

    /// Returns information about the selected physical GPU.
    pub fn selected_device_info(&self) -> &VulkanDeviceInfo { &self.selected_device_info }

    /// Returns the logical device.
    pub fn device(&self) -> &VulkanLogicalDevice { &self.device }

    /// Records an empty one-time command buffer.
    ///
    /// This exercises command-buffer allocation, recording and finalization.
    /// It does not submit work to the queue or present a frame.
    pub fn record_empty_command_buffer(
        &self,
    ) -> Result<VulkanCommandBuffer<'_>, VulkanCommandError> {
        let mut command_buffer =
            VulkanCommandBuffer::new(&self.device).map_err(VulkanCommandError)?;
        command_buffer.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .map_err(VulkanCommandError)?;
        command_buffer.end().map_err(VulkanCommandError)?;
        Ok(command_buffer)
    }

    fn build_selected_device_info(
        context: &VulkanContext,
        device: &PhysicalDevice,
    ) -> VulkanDeviceInfo {
        let has_graphics_queue = context.queue_families(device).iter()
            .any(|family| family.supports_graphics && family.queue_count > 0);
        VulkanDeviceInfo::from_device(device, has_graphics_queue)
    }
}

impl RenderBackend for VulkanBackend {
    type Error = VulkanRenderError;

    /// Output presentation awaits surface and swapchain integration.
    fn render_frame(
        &mut self,
        _target: &RenderTarget,
        _scene: &RenderScene<'_>,
    ) -> Result<(), Self::Error> {
        Err(VulkanRenderError::PresentationUnavailable)
    }
}
