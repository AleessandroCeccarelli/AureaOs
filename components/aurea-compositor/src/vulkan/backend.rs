
 // SPDX-License-Identifier: GPL-3.0-or-later
 //
 // AureaOS
 // Copyright (C) 2026 Alessandro Ceccarelli

 //! Vulkan rendering-backend initialization and renderer integration.

 use std::{error::Error, fmt};

 use smithay::{
     backend::vulkan::PhysicalDevice,
     reexports::ash::vk,
 };

 use crate::{
     render::RenderScene,
     renderer::{RenderBackend, RenderPreference, RenderTarget},
 };

 use super::{
     VulkanCommandBuffer, VulkanContext, VulkanDeviceInfo, VulkanInitError,
     VulkanLogicalDevice,
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
             Self::PresentationUnavailable => {
                 formatter.write_str(
                     "Vulkan output presentation is not implemented yet",
                 )
             }
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
     pub fn new(preference: RenderPreference) -> Result<Self, VulkanInitError> {
         let context = VulkanContext::new()?;

         let physical_device = context
             .select_device(preference)
             .ok_or_else(|| {
                 VulkanInitError::new(
                     "No graphics-capable Vulkan device found",
                 )
             })?;

         let queue_family_index = context
             .graphics_queue_family(physical_device)
             .ok_or_else(|| {
                 VulkanInitError::new(
                     "Selected device has no graphics queue",
                 )
             })?;

         let selected_device_info =
             Self::build_selected_device_info(&context, physical_device);

         let device =
             VulkanLogicalDevice::new(physical_device, queue_family_index)?;

         Ok(Self {
             context,
             selected_device_info,
             device,
         })
     }

     /// Returns the Vulkan context.
     pub fn context(&self) -> &VulkanContext {
         &self.context
     }

     /// Returns information about the selected physical GPU.
     pub fn selected_device_info(&self) -> &VulkanDeviceInfo {
         &self.selected_device_info
     }

     /// Returns the logical device.
     pub fn device(&self) -> &VulkanLogicalDevice {
         &self.device
     }

     /// Records an empty one-time command buffer.
     ///
     /// This exercises command-buffer allocation, recording and finalization.
     /// It does not submit work to the queue or present a frame.
     pub fn record_empty_command_buffer(
         &self,
     ) -> Result<VulkanCommandBuffer<'_>, VulkanCommandError> {
         let command_buffer =
             VulkanCommandBuffer::new(&self.device)
                 .map_err(VulkanCommandError)?;

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
             .any(|family| {
                 family.supports_graphics && family.queue_count > 0
             });

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
