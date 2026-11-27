// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Host-visible staging buffer for Vulkan image readback.

use std::{slice, sync::Arc};

use smithay::reexports::ash::vk;

use super::{
    VulkanLogicalDevice, VulkanReadbackError, readback_buffer_device::VulkanReadbackDevice,
};

/// Owns a host-visible buffer sized for tightly packed RGBA8 pixels.
pub struct VulkanReadbackBuffer<'device> {
    device: VulkanReadbackDevice<'device>,
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    byte_len: usize,
    allocation_size: usize,
    coherent: bool,
}

impl<'device> VulkanReadbackBuffer<'device> {
    /// Allocates a host-visible staging buffer for an RGBA8 image.
    pub fn new(
        device: &'device VulkanLogicalDevice,
        width: u32,
        height: u32,
    ) -> Result<Self, VulkanReadbackError> {
        Self::new_with_owner(VulkanReadbackDevice::Borrowed(device), width, height, None)
    }

    /// Allocates a staging buffer that independently retains its device.
    pub(super) fn new_owned(
        device: Arc<VulkanLogicalDevice>,
        width: u32,
        height: u32,
        max_allocation_size: usize,
    ) -> Result<VulkanReadbackBuffer<'static>, VulkanReadbackError> {
        VulkanReadbackBuffer::<'static>::new_with_owner(
            VulkanReadbackDevice::Shared(device),
            width,
            height,
            Some(max_allocation_size),
        )
    }

    fn new_with_owner(
        device: VulkanReadbackDevice<'device>,
        width: u32,
        height: u32,
        max_allocation_size: Option<usize>,
    ) -> Result<Self, VulkanReadbackError> {
        let raw_device = device.device();
        if width == 0 || height == 0 {
            return Err(VulkanReadbackError::InvalidDimensions);
        }
        let byte_len = (width as usize)
            .checked_mul(height as usize)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or(VulkanReadbackError::SizeOverflow)?;
        let size =
            vk::DeviceSize::try_from(byte_len).map_err(|_| VulkanReadbackError::SizeOverflow)?;

        let create_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(vk::BufferUsageFlags::TRANSFER_DST)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let buffer = unsafe { raw_device.handle().create_buffer(&create_info, None) }
            .map_err(|error| {
                if error == vk::Result::ERROR_DEVICE_LOST {
                    raw_device.mark_unusable();
                }
                VulkanReadbackError::BufferCreation(error)
            })?;
        let requirements = unsafe { raw_device.handle().get_buffer_memory_requirements(buffer) };
        let allocation_size = match usize::try_from(requirements.size) {
            Ok(size) => size,
            Err(_) => {
                unsafe { raw_device.handle().destroy_buffer(buffer, None) };
                return Err(VulkanReadbackError::SizeOverflow);
            }
        };

        if max_allocation_size.is_some_and(|limit| allocation_size > limit) {
            unsafe { raw_device.handle().destroy_buffer(buffer, None) };
            return Err(VulkanReadbackError::PoolExhausted);
        }

        let properties = raw_device.memory_properties();
        let memory_type_index = match super::select_memory_type_index(
            properties,
            requirements.memory_type_bits,
            vk::MemoryPropertyFlags::HOST_VISIBLE,
            vk::MemoryPropertyFlags::HOST_COHERENT,
        ) {
            Some(index) => index,
            None => {
                unsafe { raw_device.handle().destroy_buffer(buffer, None) };
                return Err(VulkanReadbackError::MemoryTypeUnavailable);
            }
        };
        let coherent = properties.memory_types[memory_type_index as usize]
            .property_flags
            .contains(vk::MemoryPropertyFlags::HOST_COHERENT);

        let allocate_info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type_index);
        let memory = match unsafe { raw_device.handle().allocate_memory(&allocate_info, None) } {
            Ok(memory) => memory,
            Err(error) => {
                if error == vk::Result::ERROR_DEVICE_LOST {
                    raw_device.mark_unusable();
                } else {
                    unsafe { raw_device.handle().destroy_buffer(buffer, None) };
                }
                return Err(VulkanReadbackError::MemoryAllocation(error));
            }
        };
        if let Err(error) = unsafe { raw_device.handle().bind_buffer_memory(buffer, memory, 0) } {
            if error == vk::Result::ERROR_DEVICE_LOST {
                raw_device.mark_unusable();
            } else {
                unsafe {
                    raw_device.handle().free_memory(memory, None);
                    raw_device.handle().destroy_buffer(buffer, None);
                }
            }
            return Err(VulkanReadbackError::MemoryBinding(error));
        }

        Ok(Self {
            device,
            buffer,
            memory,
            byte_len,
            allocation_size,
            coherent,
        })
    }

    /// Records a tightly packed image-to-buffer copy.
    ///
    /// The image must already be in TRANSFER_SRC_OPTIMAL and have been created
    /// with TRANSFER_SRC usage. This helper accepts only RGBA8 UNORM views.
    pub fn record_copy_rgba8(
        &self,
        command_buffer: vk::CommandBuffer,
        view: &super::VulkanImageView<'_, '_>,
    ) -> Result<(), VulkanReadbackError> {
        if view.format() != vk::Format::R8G8B8A8_UNORM {
            return Err(VulkanReadbackError::UnsupportedImageFormat);
        }
        if !view.usage().contains(vk::ImageUsageFlags::TRANSFER_SRC) {
            return Err(VulkanReadbackError::MissingTransferSourceUsage);
        }
        let expected = (view.width() as usize)
            .checked_mul(view.height() as usize)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or(VulkanReadbackError::SizeOverflow)?;
        if expected != self.byte_len {
            return Err(VulkanReadbackError::InvalidDimensions);
        }

        let region = [vk::BufferImageCopy::default()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(0)
                    .base_array_layer(0)
                    .layer_count(1),
            )
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(vk::Extent3D {
                width: view.width(),
                height: view.height(),
                depth: 1,
            })];
        unsafe {
            self.device.device().handle().cmd_copy_image_to_buffer(
                command_buffer,
                view.image_handle(),
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                self.buffer,
                &region,
            );
        }

        let buffer_barrier = [vk::BufferMemoryBarrier::default()
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::HOST_READ)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .buffer(self.buffer)
            .offset(0)
            .size(vk::WHOLE_SIZE)];
        unsafe {
            self.device.device().handle().cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::HOST,
                vk::DependencyFlags::empty(),
                &[],
                &buffer_barrier,
                &[],
            );
        }
        Ok(())
    }

    /// Records the image transition and copy required for an RGBA8 readback.
    ///
    /// The caller supplies the image's actual previous state. The image must
    /// have TRANSFER_SRC usage; after this call it remains in TRANSFER_SRC_OPTIMAL.
    /// The caller must submit and wait before mapping the buffer.
    pub fn record_transition_and_copy_rgba8(
        &self,
        device: &VulkanLogicalDevice,
        command_buffer: vk::CommandBuffer,
        view: &super::VulkanImageView<'_, '_>,
        previous_state: super::VulkanImageState,
    ) -> Result<(), VulkanReadbackError> {
        if view.format() != vk::Format::R8G8B8A8_UNORM {
            return Err(VulkanReadbackError::UnsupportedImageFormat);
        }
        if !view.usage().contains(vk::ImageUsageFlags::TRANSFER_SRC) {
            return Err(VulkanReadbackError::MissingTransferSourceUsage);
        }

        super::VulkanImageTransition::record_color(
            device,
            command_buffer,
            view.image_handle(),
            previous_state,
            super::VulkanImageState::TransferSource,
        );
        self.record_copy_rgba8(command_buffer, view)
    }

    /// Maps completed GPU output and returns tightly packed RGBA bytes.
    ///
    /// The caller must wait for the submission that wrote this buffer before
    /// calling this method. Non-coherent memory is invalidated before reading.
    pub fn read_rgba8(&self) -> Result<Vec<u8>, VulkanReadbackError> {
        let mapped = unsafe {
            self.device.device().handle().map_memory(
                self.memory,
                0,
                vk::WHOLE_SIZE,
                vk::MemoryMapFlags::empty(),
            )
        }
        .map_err(|error| {
            if error == vk::Result::ERROR_DEVICE_LOST {
                self.device.device().mark_unusable();
            }
            VulkanReadbackError::Mapping(error)
        })?;

        if !self.coherent {
            let ranges = [vk::MappedMemoryRange::default()
                .memory(self.memory)
                .offset(0)
                .size(vk::WHOLE_SIZE)];
            if let Err(error) = unsafe {
                self.device
                    .device()
                    .handle()
                    .invalidate_mapped_memory_ranges(&ranges)
            } {
                if error == vk::Result::ERROR_DEVICE_LOST {
                    self.device.device().mark_unusable();
                } else {
                    unsafe { self.device.device().handle().unmap_memory(self.memory) };
                }
                return Err(VulkanReadbackError::Invalidation(error));
            }
        }

        let bytes = unsafe { slice::from_raw_parts(mapped.cast::<u8>(), self.byte_len).to_vec() };
        unsafe {
            self.device.device().handle().unmap_memory(self.memory);
        }
        Ok(bytes)
    }

    pub fn byte_len(&self) -> usize {
        self.byte_len
    }

    pub(super) fn allocation_size(&self) -> usize {
        self.allocation_size
    }
}

impl Drop for VulkanReadbackBuffer<'_> {
    fn drop(&mut self) {
        if self.device.device().is_unusable() { return; }
        unsafe {
            self.device
                .device()
                .handle()
                .destroy_buffer(self.buffer, None);
            self.device.device().handle().free_memory(self.memory, None);
        }
    }
}
