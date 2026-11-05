
 // SPDX-License-Identifier: GPL-3.0-or-later
 //
 // AureaOS
 // Copyright (C) 2026 Alessandro Ceccarelli

 //! RAII wrapper for Vulkan command-buffer recording.

 use smithay::reexports::ash::vk;

 use super::VulkanLogicalDevice;

 /// Owns a primary command buffer allocated from a logical device.
 ///
 /// The command buffer is freed automatically when this wrapper is dropped.
 /// The caller must ensure it is not pending execution before dropping it.
 pub struct VulkanCommandBuffer<'device> {
     device: &'device VulkanLogicalDevice,
     handle: vk::CommandBuffer,
 }

 impl<'device> VulkanCommandBuffer<'device> {
     /// Allocates a primary command buffer from the device's command pool.
     pub fn new(
         device: &'device VulkanLogicalDevice,
     ) -> Result<Self, vk::Result> {
         let handle = device.allocate_primary_command_buffer()?;

         Ok(Self { device, handle })
     }

     /// Returns the underlying Vulkan command-buffer handle.
     pub fn handle(&self) -> vk::CommandBuffer {
         self.handle
     }

     /// Begins recording commands.
     pub fn begin(
         &self,
         usage: vk::CommandBufferUsageFlags,
     ) -> Result<(), vk::Result> {
         let begin_info =
             vk::CommandBufferBeginInfo::default().flags(usage);

         unsafe {
             self.device
                 .handle()
                 .begin_command_buffer(self.handle, &begin_info)
         }
     }

     /// Ends recording commands.
     pub fn end(&self) -> Result<(), vk::Result> {
         unsafe {
             self.device
                 .handle()
                 .end_command_buffer(self.handle)
         }
     }

     /// Resets the command buffer so it can be recorded again.
     ///
     /// The buffer must not be pending execution.
     pub fn reset(
         &self,
         flags: vk::CommandBufferResetFlags,
     ) -> Result<(), vk::Result> {
         unsafe {
             self.device
                 .handle()
                 .reset_command_buffer(self.handle, flags)
         }
     }
 }

 impl Drop for VulkanCommandBuffer<'_> {
     fn drop(&mut self) {
         self.device.free_command_buffer(self.handle);
     }
 }
