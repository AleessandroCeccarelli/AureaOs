
 // SPDX-License-Identifier: GPL-3.0-or-later
 //
 // AureaOS
 // Copyright (C) 2026 Alessandro Ceccarelli

 //! Vulkan logical-device and command-pool resource management.

 use smithay::{
     backend::vulkan::PhysicalDevice,
     reexports::ash::{self, vk},
 };

 use super::{VulkanContext, VulkanInitError, VulkanQueueSubmitter};

 /// A logical Vulkan device, graphics queue and command pool.
 pub struct VulkanLogicalDevice {
     device: ash::Device,
     graphics_queue: vk::Queue,
     graphics_queue_family: u32,
     command_pool: vk::CommandPool,
 }

 impl VulkanLogicalDevice {
     /// Creates a logical device with one graphics queue and command pool.
     pub(super) fn new(
         physical_device: &PhysicalDevice,
         queue_family_index: u32,
     ) -> Result<Self, VulkanInitError> {
         let priorities = [1.0_f32];
         let queue_info = [vk::DeviceQueueCreateInfo::default()
             .queue_family_index(queue_family_index)
             .queue_priorities(&priorities)];

         let create_info =
             vk::DeviceCreateInfo::default().queue_create_infos(&queue_info);

         let device = unsafe {
             physical_device
                 .instance()
                 .handle()
                 .create_device(physical_device.handle(), &create_info, None)
         }
         .map_err(|error| {
             VulkanInitError::new(format!(
                 "Failed to create Vulkan logical device: {error}"
             ))
         })?;

         let graphics_queue =
             unsafe { device.get_device_queue(queue_family_index, 0) };

         let pool_info = vk::CommandPoolCreateInfo::default()
             .queue_family_index(queue_family_index)
             .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

         let command_pool = match unsafe {
             device.create_command_pool(&pool_info, None)
         } {
             Ok(pool) => pool,
             Err(error) => {
                 unsafe {
                     device.destroy_device(None);
                 }

                 return Err(VulkanInitError::new(format!(
                     "Failed to create Vulkan command pool: {error}"
                 )));
             }
         };

         Ok(Self {
             device,
             graphics_queue,
             graphics_queue_family: queue_family_index,
             command_pool,
         })
     }

     /// Returns the underlying Vulkan device handle.
     pub fn handle(&self) -> &ash::Device {
         &self.device
     }

     /// Returns the graphics queue handle.
     pub fn graphics_queue(&self) -> vk::Queue {
         self.graphics_queue
     }

     /// Returns the graphics queue-family index.
     pub fn graphics_queue_family(&self) -> u32 {
         self.graphics_queue_family
     }

     /// Returns the command pool used by this device.
     pub fn command_pool(&self) -> vk::CommandPool {
         self.command_pool
     }

     /// Allocates one primary command buffer from this device's command pool.
     ///
     /// The caller must externally synchronize command-pool access when
     /// using the pool from multiple threads.
     pub fn allocate_primary_command_buffer(
         &self,
     ) -> Result<vk::CommandBuffer, vk::Result> {
         let allocation_info = vk::CommandBufferAllocateInfo::default()
             .command_pool(self.command_pool)
             .level(vk::CommandBufferLevel::PRIMARY)
             .command_buffer_count(1);

         let command_buffers = unsafe {
             self.device
                 .allocate_command_buffers(&allocation_info)
         }?;

         Ok(command_buffers[0])
     }

     /// Frees a primary command buffer allocated from this device's pool.
     ///
     /// The command buffer must not be pending execution.
     pub fn free_command_buffer(
         &self,
         command_buffer: vk::CommandBuffer,
     ) {
         unsafe {
             self.device.free_command_buffers(
                 self.command_pool,
                 &[command_buffer],
             );
         }
     }

     /// Submits a recorded command buffer and waits for its completion.
     pub fn submit_and_wait(
         &self,
         command_buffer: vk::CommandBuffer,
     ) -> Result<(), vk::Result> {
         VulkanQueueSubmitter::new(&self.device, self.graphics_queue)
             .submit_and_wait(command_buffer)
     }
 }

 impl Drop for VulkanLogicalDevice {
     fn drop(&mut self) {
         unsafe {
             let _ = self.device.device_wait_idle();
             self.device.destroy_command_pool(self.command_pool, None);
             self.device.destroy_device(None);
         }
     }
 }

 impl VulkanContext {
     /// Creates a logical device for the selected physical device.
     pub fn create_logical_device(
         &self,
         preference: crate::renderer::RenderPreference,
     ) -> Result<VulkanLogicalDevice, VulkanInitError> {
         let physical_device = self
             .select_device(preference)
             .ok_or_else(|| {
                 VulkanInitError::new(
                     "No graphics-capable Vulkan device found",
                 )
             })?;

         let queue_family_index = self
             .graphics_queue_family(physical_device)
             .ok_or_else(|| {
                 VulkanInitError::new(
                     "Selected device has no graphics queue",
                 )
             })?;

         VulkanLogicalDevice::new(physical_device, queue_family_index)
     }
 }
