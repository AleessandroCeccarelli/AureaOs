// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Host-visible staging buffer for uploading tightly packed RGBA8 pixels.

use std::{error::Error, fmt};

use smithay::reexports::ash::vk;

use super::{VulkanImageView, VulkanLogicalDevice, select_memory_type_index};

/// Errors returned by the Vulkan upload buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanUploadError {
    InvalidDimensions,
    SizeOverflow,
    BufferCreation(vk::Result),
    MemoryTypeUnavailable,
    MemoryAllocation(vk::Result),
    MemoryBinding(vk::Result),
    Mapping(vk::Result),
    Flush(vk::Result),
    InvalidByteLength,
    InvalidOffset,
    UnsupportedImageFormat,
    MissingTransferDestinationUsage,
    InvalidImageDimensions,
}

impl fmt::Display for VulkanUploadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions => formatter.write_str("Upload dimensions must be non-zero"),
            Self::SizeOverflow => formatter.write_str("Upload byte size overflowed"),
            Self::BufferCreation(error) => {
                write!(formatter, "Upload buffer creation failed: {error:?}")
            }
            Self::MemoryTypeUnavailable => {
                formatter.write_str("No host-visible Vulkan memory type is available")
            }
            Self::MemoryAllocation(error) => {
                write!(formatter, "Upload memory allocation failed: {error:?}")
            }
            Self::MemoryBinding(error) => {
                write!(formatter, "Upload memory binding failed: {error:?}")
            }
            Self::Mapping(error) => write!(formatter, "Upload memory mapping failed: {error:?}"),
            Self::Flush(error) => write!(formatter, "Upload memory flush failed: {error:?}"),
            Self::InvalidByteLength => {
                formatter.write_str("RGBA8 upload data has an unexpected byte length")
            }
            Self::InvalidOffset => {
                formatter.write_str("RGBA8 upload offset is invalid or out of range")
            }
            Self::UnsupportedImageFormat => {
                formatter.write_str("Upload currently supports R8G8B8A8_UNORM images only")
            }
            Self::MissingTransferDestinationUsage => {
                formatter.write_str("Upload image lacks TRANSFER_DST usage")
            }
            Self::InvalidImageDimensions => {
                formatter.write_str("Upload buffer and destination image dimensions differ")
            }
        }
    }
}

impl Error for VulkanUploadError {}

/// Owns a host-visible staging buffer sized for tightly packed RGBA8 pixels.
pub struct VulkanUploadBuffer<'device> {
    device: &'device VulkanLogicalDevice,
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    byte_len: usize,
    coherent: bool,
    width: u32,
    height: u32,
}

impl<'device> VulkanUploadBuffer<'device> {
    /// Allocates a host-visible staging buffer for an RGBA8 image.
    pub fn new(
        device: &'device VulkanLogicalDevice,
        width: u32,
        height: u32,
    ) -> Result<Self, VulkanUploadError> {
        let byte_len = Self::required_byte_len(width, height)?;
        let size =
            vk::DeviceSize::try_from(byte_len).map_err(|_| VulkanUploadError::SizeOverflow)?;

        let create_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let buffer = unsafe { device.handle().create_buffer(&create_info, None) }
            .map_err(VulkanUploadError::BufferCreation)?;
        let requirements = unsafe { device.handle().get_buffer_memory_requirements(buffer) };
        let properties = device.memory_properties();

        let memory_type_index = match select_memory_type_index(
            properties,
            requirements.memory_type_bits,
            vk::MemoryPropertyFlags::HOST_VISIBLE,
            vk::MemoryPropertyFlags::HOST_COHERENT,
        ) {
            Some(index) => index,
            None => {
                unsafe { device.handle().destroy_buffer(buffer, None) };
                return Err(VulkanUploadError::MemoryTypeUnavailable);
            }
        };
        let coherent = properties.memory_types[memory_type_index as usize]
            .property_flags
            .contains(vk::MemoryPropertyFlags::HOST_COHERENT);

        let allocate_info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type_index);
        let memory = match unsafe { device.handle().allocate_memory(&allocate_info, None) } {
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
            width,
            height,
        })
    }

    /// Calculates the tightly packed RGBA8 allocation size with overflow checks.
    pub fn required_byte_len(width: u32, height: u32) -> Result<usize, VulkanUploadError> {
        if width == 0 || height == 0 {
            return Err(VulkanUploadError::InvalidDimensions);
        }

        (width as usize)
            .checked_mul(height as usize)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or(VulkanUploadError::SizeOverflow)
    }

    /// Copies a complete RGBA8 frame into mapped staging memory.
    ///
    /// Non-coherent memory is flushed before the buffer is used by the GPU.
    pub fn write_rgba8(&self, pixels: &[u8]) -> Result<(), VulkanUploadError> {
        if pixels.len() != self.byte_len {
            return Err(VulkanUploadError::InvalidByteLength);
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

        unsafe {
            std::ptr::copy_nonoverlapping(pixels.as_ptr(), mapped.cast::<u8>(), self.byte_len);
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

    /// Records a staging-buffer upload into an RGBA8 destination image.
    ///
    /// The destination must already be in TRANSFER_DST_OPTIMAL. The caller
    /// owns image layout transitions, command submission and resource lifetime.
    pub fn record_upload_rgba8(
        &self,
        command_buffer: vk::CommandBuffer,
        destination: &VulkanImageView<'_, '_>,
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
        if destination.width() != self.width || destination.height() != self.height {
            return Err(VulkanUploadError::InvalidImageDimensions);
        }

        let buffer_barrier = [vk::BufferMemoryBarrier::default()
            .src_access_mask(vk::AccessFlags::HOST_WRITE)
            .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .buffer(self.buffer)
            .offset(0)
            .size(vk::WHOLE_SIZE)];
        unsafe {
            self.device.handle().cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::HOST,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &buffer_barrier,
                &[],
            );
        }

        let regions = [vk::BufferImageCopy::default()
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
                width: self.width,
                height: self.height,
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

    pub fn byte_len(&self) -> usize {
        self.byte_len
    }
}

impl Drop for VulkanUploadBuffer<'_> {
    fn drop(&mut self) {
        unsafe {
            self.device.handle().destroy_buffer(self.buffer, None);
            self.device.handle().free_memory(self.memory, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_tightly_packed_rgba8_size() {
        assert_eq!(VulkanUploadBuffer::required_byte_len(8, 4), Ok(128));
    }

    #[test]
    fn rejects_zero_dimensions() {
        assert_eq!(
            VulkanUploadBuffer::required_byte_len(0, 4),
            Err(VulkanUploadError::InvalidDimensions)
        );
    }

    #[test]
    fn rejects_size_overflow() {
        assert_eq!(
            VulkanUploadBuffer::required_byte_len(u32::MAX, u32::MAX),
            Err(VulkanUploadError::SizeOverflow)
        );
    }
}
