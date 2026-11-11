// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Native Vulkan image allocation on AureaOS-owned logical devices.

use std::{error::Error, fmt};

use smithay::reexports::ash::vk;

use super::{VulkanLogicalDevice, memory_type::select_memory_type_index};

/// Description of a two-dimensional Vulkan image allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VulkanImageRequest {
    pub width: u32,
    pub height: u32,
    pub format: vk::Format,
    pub usage: vk::ImageUsageFlags,
    pub tiling: vk::ImageTiling,
    pub required_memory: vk::MemoryPropertyFlags,
    pub preferred_memory: vk::MemoryPropertyFlags,
}

impl VulkanImageRequest {
    /// Creates a device-local, optimal-tiled 2D image request.
    pub fn render_target(
        width: u32,
        height: u32,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
    ) -> Self {
        Self {
            width,
            height,
            format,
            usage,
            tiling: vk::ImageTiling::OPTIMAL,
            required_memory: vk::MemoryPropertyFlags::DEVICE_LOCAL,
            preferred_memory: vk::MemoryPropertyFlags::DEVICE_LOCAL,
        }
    }
}

/// Errors returned while creating a native Vulkan image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanImageError {
    InvalidDimensions,
    EmptyUsage,
    ImageCreation(vk::Result),
    MemoryTypeUnavailable,
    MemoryAllocation(vk::Result),
    MemoryBinding(vk::Result),
}

impl fmt::Display for VulkanImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions => f.write_str("Vulkan image dimensions must be non-zero"),
            Self::EmptyUsage => f.write_str("Vulkan image usage flags must not be empty"),
            Self::ImageCreation(error) => write!(f, "Vulkan image creation failed: {error:?}"),
            Self::MemoryTypeUnavailable => f.write_str("No Vulkan memory type satisfies the image requirements"),
            Self::MemoryAllocation(error) => write!(f, "Vulkan image memory allocation failed: {error:?}"),
            Self::MemoryBinding(error) => write!(f, "Vulkan image memory binding failed: {error:?}"),
        }
    }
}

impl Error for VulkanImageError {}

/// Allocates images using the compositor's existing logical Vulkan device.
pub struct VulkanImageAllocator<'device> {
    device: &'device VulkanLogicalDevice,
}

impl<'device> VulkanImageAllocator<'device> {
    pub fn new(device: &'device VulkanLogicalDevice) -> Self {
        Self { device }
    }

    /// Creates and binds an image. The returned resource destroys its image
    /// before releasing the bound memory.
    pub fn allocate(&self, request: VulkanImageRequest) -> Result<VulkanImage<'device>, VulkanImageError> {
        if request.width == 0 || request.height == 0 {
            return Err(VulkanImageError::InvalidDimensions);
        }
        if request.usage.is_empty() {
            return Err(VulkanImageError::EmptyUsage);
        }

        let create_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(request.format)
            .extent(vk::Extent3D {
                width: request.width,
                height: request.height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(request.tiling)
            .usage(request.usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let raw = self.device.handle();
        let image = unsafe { raw.create_image(&create_info, None) }
            .map_err(VulkanImageError::ImageCreation)?;
        let requirements = unsafe { raw.get_image_memory_requirements(image) };

        let memory_type_index = match select_memory_type_index(
            self.device.memory_properties(),
            requirements.memory_type_bits,
            request.required_memory,
            request.preferred_memory,
        ) {
            Some(index) => index,
            None => {
                unsafe { raw.destroy_image(image, None) };
                return Err(VulkanImageError::MemoryTypeUnavailable);
            }
        };

        let allocate_info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type_index);
        let memory = match unsafe { raw.allocate_memory(&allocate_info, None) } {
            Ok(memory) => memory,
            Err(error) => {
                unsafe { raw.destroy_image(image, None) };
                return Err(VulkanImageError::MemoryAllocation(error));
            }
        };

        if let Err(error) = unsafe { raw.bind_image_memory(image, memory, 0) } {
            unsafe {
                raw.free_memory(memory, None);
                raw.destroy_image(image, None);
            }
            return Err(VulkanImageError::MemoryBinding(error));
        }

        Ok(VulkanImage {
            device: self.device,
            image,
            memory,
            width: request.width,
            height: request.height,
            format: request.format,
            usage: request.usage,
        })
    }
}

/// Owns a Vulkan image and the device memory bound to it.
pub struct VulkanImage<'device> {
    device: &'device VulkanLogicalDevice,
    image: vk::Image,
    memory: vk::DeviceMemory,
    width: u32,
    height: u32,
    format: vk::Format,
    usage: vk::ImageUsageFlags,
}

impl<'device> VulkanImage<'device> {
    pub fn handle(&self) -> vk::Image { self.image }
    pub fn memory(&self) -> vk::DeviceMemory { self.memory }
    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    pub fn format(&self) -> vk::Format { self.format }
    pub fn usage(&self) -> vk::ImageUsageFlags { self.usage }
    pub(super) fn device(&self) -> &'device VulkanLogicalDevice { self.device }
}

impl Drop for VulkanImage<'_> {
    fn drop(&mut self) {
        let raw = self.device.handle();
        unsafe {
            raw.destroy_image(self.image, None);
            raw.free_memory(self.memory, None);
        }
    }
}
