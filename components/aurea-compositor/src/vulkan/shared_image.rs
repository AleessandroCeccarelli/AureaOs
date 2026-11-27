// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan images that retain shared ownership of their logical device.

use std::{error::Error, fmt, sync::Arc};

use smithay::reexports::ash::vk;

use super::{VulkanLogicalDevice, memory_type::select_memory_type_index};

/// Errors returned while allocating a device-owned intermediate image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanSharedImageError {
    InvalidDimensions,
    EmptyUsage,
    ImageCreation(vk::Result),
    MemoryTypeUnavailable,
    MemoryAllocation(vk::Result),
    MemoryBinding(vk::Result),
}

impl fmt::Display for VulkanSharedImageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions => {
                formatter.write_str("Vulkan image dimensions must be non-zero")
            }
            Self::EmptyUsage => formatter.write_str("Vulkan image usage flags must not be empty"),
            Self::ImageCreation(error) => {
                write!(formatter, "Vulkan image creation failed: {error:?}")
            }
            Self::MemoryTypeUnavailable => {
                formatter.write_str("No compatible device-local memory type is available")
            }
            Self::MemoryAllocation(error) => write!(
                formatter,
                "Vulkan image memory allocation failed: {error:?}"
            ),
            Self::MemoryBinding(error) => {
                write!(formatter, "Vulkan image memory binding failed: {error:?}")
            }
        }
    }
}

impl Error for VulkanSharedImageError {}

/// Owns an optimal-tiled image and keeps its logical device alive.
///
/// Unlike the short-lived borrowed image allocator, this resource can safely
/// live inside a DRM frame pool whose lifetime is independent of a stack frame.
pub struct VulkanSharedImage {
    device: Arc<VulkanLogicalDevice>,
    image: vk::Image,
    memory: vk::DeviceMemory,
    width: u32,
    height: u32,
    format: vk::Format,
    usage: vk::ImageUsageFlags,
    allocation_size: usize,
}

impl VulkanSharedImage {
    pub fn allocate(
        device: Arc<VulkanLogicalDevice>,
        width: u32,
        height: u32,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
    ) -> Result<Self, VulkanSharedImageError> {
        if width == 0 || height == 0 {
            return Err(VulkanSharedImageError::InvalidDimensions);
        }
        if usage.is_empty() {
            return Err(VulkanSharedImageError::EmptyUsage);
        }

        let request = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let raw = device.handle();
        let image = unsafe { raw.create_image(&request, None) }
            .map_err(VulkanSharedImageError::ImageCreation)?;
        let requirements = unsafe { raw.get_image_memory_requirements(image) };
        let allocation_size = match usize::try_from(requirements.size) {
            Ok(size) => size,
            Err(_) => {
                unsafe { raw.destroy_image(image, None) };
                return Err(VulkanSharedImageError::MemoryAllocation(
                    vk::Result::ERROR_OUT_OF_HOST_MEMORY,
                ));
            }
        };
        let memory_type = select_memory_type_index(
            device.memory_properties(),
            requirements.memory_type_bits,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );
        let Some(memory_type_index) = memory_type else {
            unsafe { raw.destroy_image(image, None) };
            return Err(VulkanSharedImageError::MemoryTypeUnavailable);
        };

        let allocation = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type_index);
        let memory = match unsafe { raw.allocate_memory(&allocation, None) } {
            Ok(memory) => memory,
            Err(error) => {
                unsafe { raw.destroy_image(image, None) };
                return Err(VulkanSharedImageError::MemoryAllocation(error));
            }
        };

        if let Err(error) = unsafe { raw.bind_image_memory(image, memory, 0) } {
            unsafe {
                raw.free_memory(memory, None);
                raw.destroy_image(image, None);
            }
            return Err(VulkanSharedImageError::MemoryBinding(error));
        }

        Ok(Self {
            device,
            image,
            memory,
            width,
            height,
            format,
            usage,
            allocation_size,
        })
    }

    pub fn handle(&self) -> vk::Image {
        self.image
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn format(&self) -> vk::Format {
        self.format
    }
    pub fn usage(&self) -> vk::ImageUsageFlags {
        self.usage
    }
    pub fn allocation_size(&self) -> usize {
        self.allocation_size
    }

    pub(super) fn device(&self) -> &VulkanLogicalDevice {
        self.device.as_ref()
    }

    pub(super) fn device_owner(&self) -> Arc<VulkanLogicalDevice> {
        Arc::clone(&self.device)
    }
}

impl Drop for VulkanSharedImage {
    fn drop(&mut self) {
        if self.device.is_unusable() { return; }
        let raw = self.device.handle();
        unsafe {
            raw.destroy_image(self.image, None);
            raw.free_memory(self.memory, None);
        }
    }
}
