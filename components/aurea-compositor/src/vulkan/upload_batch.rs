// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Shared host-visible staging allocation for a batch of RGBA8 texture uploads.

use smithay::reexports::ash::vk;

use super::{
    select_memory_type_index, VulkanImageView, VulkanLogicalDevice, VulkanUploadBuffer,
    VulkanUploadError,
};

/// One staging allocation shared by all texture copies in a submission.
pub(super) struct VulkanUploadBatch<'device> {
    device: &'device VulkanLogicalDevice,
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    byte_len: usize,
    coherent: bool,
}

impl<'device> VulkanUploadBatch<'device> {
    pub(super) fn new(
        device: &'device VulkanLogicalDevice,
        byte_len: usize,
    ) -> Result<Self, VulkanUploadError> {
        if byte_len == 0 {
            return Err(VulkanUploadError::InvalidByteLength);
        }
        let size =
            vk::DeviceSize::try_from(byte_len).map_err(|_| VulkanUploadError::SizeOverflow)?;
        let info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let buffer = unsafe { device.handle().create_buffer(&info, None) }
            .map_err(VulkanUploadError::BufferCreation)?;
        let requirements = unsafe { device.handle().get_buffer_memory_requirements(buffer) };
        let properties = device.memory_properties();
        let memory_type = select_memory_type_index(
            properties,
            requirements.memory_type_bits,
            vk::MemoryPropertyFlags::HOST_VISIBLE,
            vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        let Some(memory_type_index) = memory_type else {
            unsafe { device.handle().destroy_buffer(buffer, None) };
            return Err(VulkanUploadError::MemoryTypeUnavailable);
        };
        let coherent = properties.memory_types[memory_type_index as usize]
            .property_flags
            .contains(vk::MemoryPropertyFlags::HOST_COHERENT);
        let allocation = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type_index);
        let memory = match unsafe { device.handle().allocate_memory(&allocation, None) } {
            Ok(memory) => memory,
            Err(error) => {
                unsafe { device.handle().destroy_buffer(buffer, None) };
                return Err(VulkanUploadError::MemoryAllocation(error));
            }
        };
        if let Err(error) = unsafe { device.handle().bind_buffer_memory(buffer, memory, 0) } {
            unsafe {
                device.handle().free_memory(memory, None);
                device.handle().destroy_buffer(buffer, None);
            }
            return Err(VulkanUploadError::MemoryBinding(error));
        }

        Ok(Self {
            device,
            buffer,
            memory,
            byte_len,
            coherent,
        })
    }

    /// Writes every image segment through one mapping and one cache flush.
    pub(super) fn write_segments(
        &self,
        segments: &[(usize, &[u8])],
    ) -> Result<(), VulkanUploadError> {
        for (offset, pixels) in segments {
            let end = offset
                .checked_add(pixels.len())
                .ok_or(VulkanUploadError::SizeOverflow)?;
            if offset % 4 != 0 || end > self.byte_len {
                return Err(VulkanUploadError::InvalidOffset);
            }
        }

        let mapped = unsafe {
            self.device.handle().map_memory(
                self.memory,
                0,
                vk::WHOLE_SIZE,
                vk::MemoryMapFlags::empty(),
            )
        }
        .map_err(VulkanUploadError::Mapping)?;

        for (offset, pixels) in segments {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    pixels.as_ptr(),
                    mapped.cast::<u8>().add(*offset),
                    pixels.len(),
                );
            }
        }

        if !self.coherent {
            let ranges = [vk::MappedMemoryRange::default()
                .memory(self.memory)
                .offset(0)
                .size(vk::WHOLE_SIZE)];
            if let Err(error) = unsafe { self.device.handle().flush_mapped_memory_ranges(&ranges) }
            {
                unsafe { self.device.handle().unmap_memory(self.memory) };
                return Err(VulkanUploadError::Flush(error));
            }
        }

        unsafe { self.device.handle().unmap_memory(self.memory) };
        Ok(())
    }

    /// Makes all host writes visible to subsequent transfer reads.
    pub(super) fn record_host_write_barrier(&self, command_buffer: vk::CommandBuffer) {
        let barriers = [vk::BufferMemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::HOST)
            .src_access_mask(vk::AccessFlags2::HOST_WRITE)
            .dst_stage_mask(vk::PipelineStageFlags2::TRANSFER)
            .dst_access_mask(vk::AccessFlags2::TRANSFER_READ)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .buffer(self.buffer)
            .offset(0)
            .size(vk::WHOLE_SIZE)];
        let dependency = vk::DependencyInfo::default().buffer_memory_barriers(&barriers);
        unsafe {
            self.device
                .handle()
                .cmd_pipeline_barrier2(command_buffer, &dependency);
        }
    }

    /// Records one buffer-to-image copy at the supplied staging offset.
    pub(super) fn record_upload_rgba8(
        &self,
        command_buffer: vk::CommandBuffer,
        destination: &VulkanImageView<'_, '_>,
        offset: usize,
    ) -> Result<(), VulkanUploadError> {
        if destination.format() != vk::Format::R8G8B8A8_UNORM {
            return Err(VulkanUploadError::UnsupportedImageFormat);
        }
        if !destination
            .usage()
            .contains(vk::ImageUsageFlags::TRANSFER_DST)
        {
            return Err(VulkanUploadError::MissingTransferDestinationUsage);
        }

        let byte_len =
            VulkanUploadBuffer::required_byte_len(destination.width(), destination.height())?;
        let end = offset
            .checked_add(byte_len)
            .ok_or(VulkanUploadError::SizeOverflow)?;
        if offset % 4 != 0 || end > self.byte_len {
            return Err(VulkanUploadError::InvalidOffset);
        }
        let buffer_offset =
            vk::DeviceSize::try_from(offset).map_err(|_| VulkanUploadError::SizeOverflow)?;

        let regions = [vk::BufferImageCopy::default()
            .buffer_offset(buffer_offset)
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
                width: destination.width(),
                height: destination.height(),
                depth: 1,
            })];
        unsafe {
            self.device.handle().cmd_copy_buffer_to_image(
                command_buffer,
                self.buffer,
                destination.image_handle(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &regions,
            );
        }
        Ok(())
    }
}

impl Drop for VulkanUploadBatch<'_> {
    fn drop(&mut self) {
        unsafe {
            self.device.handle().destroy_buffer(self.buffer, None);
            self.device.handle().free_memory(self.memory, None);
        }
    }
}
