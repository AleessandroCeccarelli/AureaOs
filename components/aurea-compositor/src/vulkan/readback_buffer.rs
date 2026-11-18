// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Host-visible staging buffer for Vulkan image readback.

use std::{error::Error, fmt, slice};

use smithay::reexports::ash::vk;

use super::VulkanLogicalDevice;

/// Errors returned by the offscreen readback buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanReadbackError {
    InvalidDimensions,
    SizeOverflow,
    BufferCreation(vk::Result),
    MemoryTypeUnavailable,
    MemoryAllocation(vk::Result),
    MemoryBinding(vk::Result),
    Mapping(vk::Result),
    Invalidation(vk::Result),
    UnsupportedImageFormat,
    MissingTransferSourceUsage,
    CommandBuffer(vk::Result),
    FenceCreation(vk::Result),
    Submission(vk::Result),
    SubmissionUncertain(vk::Result),
    FenceStatus(vk::Result),
    SubmissionNotComplete,
    PixelsAlreadyRead,
}

impl fmt::Display for VulkanReadbackError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions => formatter.write_str("Readback dimensions must be non-zero"),
            Self::SizeOverflow => formatter.write_str("Readback byte size overflowed"),
            Self::BufferCreation(error) => {
                write!(formatter, "Readback buffer creation failed: {error:?}")
            }
            Self::MemoryTypeUnavailable => {
                formatter.write_str("No host-visible Vulkan memory type is available")
            }
            Self::MemoryAllocation(error) => {
                write!(formatter, "Readback memory allocation failed: {error:?}")
            }
            Self::MemoryBinding(error) => {
                write!(formatter, "Readback memory binding failed: {error:?}")
            }
            Self::Mapping(error) => write!(formatter, "Readback memory mapping failed: {error:?}"),
            Self::Invalidation(error) => {
                write!(formatter, "Readback memory invalidation failed: {error:?}")
            }
            Self::UnsupportedImageFormat => {
                formatter.write_str("Readback currently supports R8G8B8A8_UNORM images only")
            }
            Self::MissingTransferSourceUsage => {
                formatter.write_str("Readback image lacks TRANSFER_SRC usage")
            }
            Self::CommandBuffer(error) => write!(
                formatter,
                "Readback command-buffer operation failed: {error:?}"
            ),
            Self::FenceCreation(error) => {
                write!(formatter, "Readback fence creation failed: {error:?}")
            }
            Self::Submission(error) => {
                write!(formatter, "Readback queue submission failed: {error:?}")
            }
            Self::SubmissionUncertain(error) => write!(
                formatter,
                "Readback queue submission status is uncertain: {error:?}"
            ),
            Self::FenceStatus(error) => {
                write!(formatter, "Readback fence status query failed: {error:?}")
            }
            Self::SubmissionNotComplete => {
                formatter.write_str("Readback submission has not completed")
            }
            Self::PixelsAlreadyRead => {
                formatter.write_str("Readback pixels have already been consumed")
            }
        }
    }
}

impl Error for VulkanReadbackError {}

/// Owns a host-visible buffer sized for tightly packed RGBA8 pixels.
pub struct VulkanReadbackBuffer<'device> {
    device: &'device VulkanLogicalDevice,
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    byte_len: usize,
    coherent: bool,
}

impl<'device> VulkanReadbackBuffer<'device> {
    /// Allocates a host-visible staging buffer for an RGBA8 image.
    pub fn new(
        device: &'device VulkanLogicalDevice,
        width: u32,
        height: u32,
    ) -> Result<Self, VulkanReadbackError> {
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
        let buffer = unsafe { device.handle().create_buffer(&create_info, None) }
            .map_err(VulkanReadbackError::BufferCreation)?;
        let requirements = unsafe { device.handle().get_buffer_memory_requirements(buffer) };

        let properties = device.memory_properties();
        let memory_type_index = match super::select_memory_type_index(
            properties,
            requirements.memory_type_bits,
            vk::MemoryPropertyFlags::HOST_VISIBLE,
            vk::MemoryPropertyFlags::HOST_COHERENT,
        ) {
            Some(index) => index,
            None => {
                unsafe { device.handle().destroy_buffer(buffer, None) };
                return Err(VulkanReadbackError::MemoryTypeUnavailable);
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
                return Err(VulkanReadbackError::MemoryAllocation(error));
            }
        };
        if let Err(error) = unsafe { device.handle().bind_buffer_memory(buffer, memory, 0) } {
            unsafe {
                device.handle().free_memory(memory, None);
                device.handle().destroy_buffer(buffer, None);
            }
            return Err(VulkanReadbackError::MemoryBinding(error));
        }

        Ok(Self {
            device,
            buffer,
            memory,
            byte_len,
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
            self.device.handle().cmd_copy_image_to_buffer(
                command_buffer,
                view.image_handle(),
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                self.buffer,
                &region,
            );
        }

        let buffer_barrier = [vk::BufferMemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
            .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
            .dst_stage_mask(vk::PipelineStageFlags2::HOST)
            .dst_access_mask(vk::AccessFlags2::HOST_READ)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .buffer(self.buffer)
            .offset(0)
            .size(vk::WHOLE_SIZE)];
        let dependency = vk::DependencyInfo::default().buffer_memory_barriers(&buffer_barrier);
        unsafe {
            self.device
                .handle()
                .cmd_pipeline_barrier2(command_buffer, &dependency);
        }
        Ok(())
    }

    /// Maps completed GPU output and returns tightly packed RGBA bytes.
    ///
    /// The caller must wait for the submission that wrote this buffer before
    /// calling this method. Non-coherent memory is invalidated before reading.
    pub fn read_rgba8(&self) -> Result<Vec<u8>, VulkanReadbackError> {
        let mapped = unsafe {
            self.device.handle().map_memory(
                self.memory,
                0,
                vk::WHOLE_SIZE,
                vk::MemoryMapFlags::empty(),
            )
        }
        .map_err(VulkanReadbackError::Mapping)?;

        if !self.coherent {
            let ranges = [vk::MappedMemoryRange::default()
                .memory(self.memory)
                .offset(0)
                .size(vk::WHOLE_SIZE)];
            if let Err(error) = unsafe {
                self.device
                    .handle()
                    .invalidate_mapped_memory_ranges(&ranges)
            } {
                unsafe { self.device.handle().unmap_memory(self.memory) };
                return Err(VulkanReadbackError::Invalidation(error));
            }
        }

        let bytes = unsafe { slice::from_raw_parts(mapped.cast::<u8>(), self.byte_len).to_vec() };
        unsafe {
            self.device.handle().unmap_memory(self.memory);
        }
        Ok(bytes)
    }

    pub fn byte_len(&self) -> usize {
        self.byte_len
    }
}

impl Drop for VulkanReadbackBuffer<'_> {
    fn drop(&mut self) {
        unsafe {
            self.device.handle().destroy_buffer(self.buffer, None);
            self.device.handle().free_memory(self.memory, None);
        }
    }
}

/// Owns an in-flight image readback without blocking the submitting thread.
///
/// The source image must be in TRANSFER_SRC_OPTIMAL and remain alive until
/// the returned submission reports completion. Poll this object from a
/// capture worker and retain it while the GPU is using it.
pub struct VulkanReadbackSubmission<'view, 'image, 'view_device> {
    device: &'view VulkanLogicalDevice,
    _source_view: &'view super::VulkanImageView<'image, 'view_device>,
    readback: Option<VulkanReadbackBuffer<'view>>,
    command_buffer: vk::CommandBuffer,
    fence: vk::Fence,
    completed: bool,
    pixels_read: bool,
}

impl<'view, 'image, 'view_device> VulkanReadbackSubmission<'view, 'image, 'view_device> {
    /// Records and submits an image copy. Returns after queue acceptance.
    /// The submission retains the source view until the GPU has finished copying.
    pub fn submit(
        view: &'view super::VulkanImageView<'image, 'view_device>,
    ) -> Result<Self, VulkanReadbackError> {
        let device = view.device();
        let readback = VulkanReadbackBuffer::new(device, view.width(), view.height())?;
        let command_buffer = device
            .allocate_primary_command_buffer()
            .map_err(VulkanReadbackError::CommandBuffer)?;

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        if let Err(error) = unsafe {
            device
                .handle()
                .begin_command_buffer(command_buffer, &begin_info)
        } {
            device.free_command_buffer(command_buffer);
            return Err(VulkanReadbackError::CommandBuffer(error));
        }

        if let Err(error) = readback.record_copy_rgba8(command_buffer, view) {
            unsafe {
                device
                    .handle()
                    .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                    .ok();
            }
            device.free_command_buffer(command_buffer);
            return Err(error);
        }

        if let Err(error) = unsafe { device.handle().end_command_buffer(command_buffer) } {
            device.free_command_buffer(command_buffer);
            return Err(VulkanReadbackError::CommandBuffer(error));
        }

        let fence_info = vk::FenceCreateInfo::default();
        let fence = match unsafe { device.handle().create_fence(&fence_info, None) } {
            Ok(fence) => fence,
            Err(error) => {
                device.free_command_buffer(command_buffer);
                return Err(VulkanReadbackError::FenceCreation(error));
            }
        };

        if let Err(error) = device.submit_async(command_buffer, fence) {
            if error.is_uncertain() {
                // Queue acceptance is uncertain (typically device loss).
                // Retain all potentially in-flight resources rather than
                // freeing memory that submitted work may still reference.
                std::mem::forget(readback);
                return Err(VulkanReadbackError::SubmissionUncertain(error.result()));
            }

            unsafe { device.handle().destroy_fence(fence, None) };
            device.free_command_buffer(command_buffer);
            return Err(VulkanReadbackError::Submission(error.result()));
        }

        Ok(Self {
            device,
            _source_view: view,
            readback: Some(readback),
            command_buffer,
            fence,
            completed: false,
            pixels_read: false,
        })
    }

    /// Checks completion without waiting. False means the GPU is still working.
    pub fn poll(&mut self) -> Result<bool, VulkanReadbackError> {
        if self.completed {
            return Ok(true);
        }

        match unsafe { self.device.handle().get_fence_status(self.fence) } {
            Ok(true) => {
                self.completed = true;
                Ok(true)
            }
            Ok(false) | Err(vk::Result::NOT_READY) => Ok(false),
            Err(error) => Err(VulkanReadbackError::FenceStatus(error)),
        }
    }

    /// Consumes completed pixel data. This method never waits for the GPU.
    pub fn take_rgba8(&mut self) -> Result<Vec<u8>, VulkanReadbackError> {
        if !self.poll()? {
            return Err(VulkanReadbackError::SubmissionNotComplete);
        }
        if self.pixels_read {
            return Err(VulkanReadbackError::PixelsAlreadyRead);
        }

        let pixels = self
            .readback
            .as_ref()
            .ok_or(VulkanReadbackError::PixelsAlreadyRead)?
            .read_rgba8()?;
        self.pixels_read = true;
        Ok(pixels)
    }

    pub const fn is_complete(&self) -> bool {
        self.completed
    }

    pub fn byte_len(&self) -> usize {
        self.readback
            .as_ref()
            .map_or(0, VulkanReadbackBuffer::byte_len)
    }
}

impl Drop for VulkanReadbackSubmission<'_, '_, '_> {
    fn drop(&mut self) {
        if !self.completed {
            let wait = unsafe {
                self.device
                    .handle()
                    .wait_for_fences(&[self.fence], true, u64::MAX)
            };
            if wait.is_err() && unsafe { self.device.handle().device_wait_idle() }.is_err() {
                // Completion is uncertain after device loss. Leak in-flight
                // resources rather than free memory potentially used by GPU.
                if let Some(readback) = self.readback.take() {
                    std::mem::forget(readback);
                }
                return;
            }
        }

        unsafe { self.device.handle().destroy_fence(self.fence, None) };
        self.device.free_command_buffer(self.command_buffer);
    }
}
