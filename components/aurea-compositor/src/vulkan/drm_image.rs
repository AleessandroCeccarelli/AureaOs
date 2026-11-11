// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! DRM-modifier Vulkan image allocation and DMA-BUF export.

use std::{
    error::Error,
    fmt,
    os::fd::{FromRawFd, OwnedFd},
};

use smithay::{
    backend::allocator::vulkan::format::get_vk_format,
    reexports::ash::{khr, vk},
};

use super::{
    DrmVulkanFormat,
    VulkanLogicalDevice,
    memory_type::select_memory_type_index,
};

/// Layout of one Vulkan DRM memory plane in the exported DMA-BUF.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VulkanDrmPlaneLayout {
    pub offset: u64,
    pub stride: u64,
    pub size: u64,
}

/// Errors returned by modifier-aware image allocation/export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanDrmImageError {
    MissingInteropExtensions,
    UnsupportedFourcc,
    InvalidDimensions,
    InvalidPlaneCount,
    InvalidPlaneLayout,
    ImageDimensionsUnsupported,
    DmabufExportUnsupported,
    ModifierImageQuery(vk::Result),
    ImageCreation(vk::Result),
    MemoryTypeUnavailable,
    MemoryAllocation(vk::Result),
    MemoryBinding(vk::Result),
    DmabufExport(vk::Result),
}

impl fmt::Display for VulkanDrmImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingInteropExtensions => f.write_str("Vulkan DMA-BUF interoperability extensions are unavailable"),
            Self::UnsupportedFourcc => f.write_str("DRM FourCC has no Vulkan format mapping"),
            Self::InvalidDimensions => f.write_str("DRM image dimensions must be non-zero"),
            Self::InvalidPlaneCount => f.write_str("DRM modifier memory-plane count is invalid"),
            Self::InvalidPlaneLayout => f.write_str("Vulkan returned an invalid or out-of-bounds DRM memory-plane layout"),
            Self::ImageDimensionsUnsupported => f.write_str("Requested dimensions exceed Vulkan image-format limits"),
            Self::DmabufExportUnsupported => f.write_str("Vulkan does not report DMA-BUF external-memory export support for this image configuration"),
            Self::ModifierImageQuery(error) => write!(f, "DRM modifier image-format query failed: {error:?}"),
            Self::ImageCreation(error) => write!(f, "DRM modifier image creation failed: {error:?}"),
            Self::MemoryTypeUnavailable => f.write_str("No compatible device-local memory type is available"),
            Self::MemoryAllocation(error) => write!(f, "Exportable Vulkan memory allocation failed: {error:?}"),
            Self::MemoryBinding(error) => write!(f, "DRM image memory binding failed: {error:?}"),
            Self::DmabufExport(error) => write!(f, "DMA-BUF FD export failed: {error:?}"),
        }
    }
}

impl Error for VulkanDrmImageError {}

/// Allocates scan-out candidates using an exact DRM FourCC/modifier pair.
pub struct VulkanDrmImageAllocator<'device> {
    device: &'device VulkanLogicalDevice,
}

impl<'device> VulkanDrmImageAllocator<'device> {
    pub fn new(device: &'device VulkanLogicalDevice) -> Result<Self, VulkanDrmImageError> {
        if !device.extension_plan().supports_dmabuf_interop() {
            return Err(VulkanDrmImageError::MissingInteropExtensions);
        }
        Ok(Self { device })
    }

    /// Creates an image with the exact negotiated modifier and exports its
    /// bound memory as one DMA-BUF FD. Every memory-plane layout is queried
    /// from Vulkan; no stride or offset is guessed by AureaOS.
    pub fn allocate_exportable(
        &self,
        width: u32,
        height: u32,
        format: DrmVulkanFormat,
        usage: vk::ImageUsageFlags,
    ) -> Result<VulkanDrmImage<'device>, VulkanDrmImageError> {
        if width == 0 || height == 0 {
            return Err(VulkanDrmImageError::InvalidDimensions);
        }
        if format.memory_plane_count == 0 || format.memory_plane_count > 4 {
            return Err(VulkanDrmImageError::InvalidPlaneCount);
        }
        let vk_format = get_vk_format(format.code).ok_or(VulkanDrmImageError::UnsupportedFourcc)?;
        let modifier = u64::from(format.modifier);

        let mut modifier_query = vk::PhysicalDeviceImageDrmFormatModifierInfoEXT::default()
            .drm_format_modifier(modifier);
        let mut external_query = vk::PhysicalDeviceExternalImageFormatInfo::default()
            .handle_type(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
        let image_query = vk::PhysicalDeviceImageFormatInfo2::default()
            .format(vk_format)
            .ty(vk::ImageType::TYPE_2D)
            .tiling(vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT)
            .usage(usage)
            .push_next(&mut modifier_query)
            .push_next(&mut external_query);
        let mut external_properties = vk::ExternalImageFormatProperties::default();
        let mut image_properties = vk::ImageFormatProperties2::default()
            .push_next(&mut external_properties);
        unsafe {
            self.device.instance().get_physical_device_image_format_properties2(
                self.device.physical_device(),
                &image_query,
                &mut image_properties,
            )
        }.map_err(VulkanDrmImageError::ModifierImageQuery)?;
        let limits = image_properties.image_format_properties;
        drop(image_properties);
        let external_memory = external_properties.external_memory_properties;
        let dma_buf_handle = vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT;
        if !external_memory
            .external_memory_features
            .contains(vk::ExternalMemoryFeatureFlags::EXPORTABLE)
            || !external_memory.compatible_handle_types.contains(dma_buf_handle)
        {
            return Err(VulkanDrmImageError::DmabufExportUnsupported);
        }
        if width > limits.max_extent.width || height > limits.max_extent.height {
            return Err(VulkanDrmImageError::ImageDimensionsUnsupported);
        }

        let modifiers = [modifier];
        let mut modifier_list = vk::ImageDrmFormatModifierListCreateInfoEXT::default()
            .drm_format_modifiers(&modifiers);
        let mut external_image = vk::ExternalMemoryImageCreateInfo::default()
            .handle_types(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk_format)
            .extent(vk::Extent3D { width, height, depth: 1 })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .push_next(&mut modifier_list)
            .push_next(&mut external_image);

        let raw = self.device.handle();
        let image = unsafe { raw.create_image(&image_info, None) }
            .map_err(VulkanDrmImageError::ImageCreation)?;
        let requirements = unsafe { raw.get_image_memory_requirements(image) };
        let memory_type_index = match select_memory_type_index(
            self.device.memory_properties(),
            requirements.memory_type_bits,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        ) {
            Some(index) => index,
            None => {
                unsafe { raw.destroy_image(image, None) };
                return Err(VulkanDrmImageError::MemoryTypeUnavailable);
            }
        };

        let mut export_info = vk::ExportMemoryAllocateInfo::default()
            .handle_types(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
        let mut dedicated_info = vk::MemoryDedicatedAllocateInfo::default().image(image);
        let allocation_info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type_index)
            .push_next(&mut export_info)
            .push_next(&mut dedicated_info);

        let memory = match unsafe { raw.allocate_memory(&allocation_info, None) } {
            Ok(memory) => memory,
            Err(error) => {
                unsafe { raw.destroy_image(image, None) };
                return Err(VulkanDrmImageError::MemoryAllocation(error));
            }
        };

        if let Err(error) = unsafe { raw.bind_image_memory(image, memory, 0) } {
            unsafe {
                raw.free_memory(memory, None);
                raw.destroy_image(image, None);
            }
            return Err(VulkanDrmImageError::MemoryBinding(error));
        }

        let fd_info = vk::MemoryGetFdInfoKHR::default()
            .memory(memory)
            .handle_type(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
        let fd_loader = khr::external_memory_fd::Device::new(
            self.device.instance(),
            raw,
        );
        let raw_fd = match unsafe { fd_loader.get_memory_fd(&fd_info) } {
            Ok(fd) => fd,
            Err(error) => {
                unsafe {
                    raw.destroy_image(image, None);
                    raw.free_memory(memory, None);
                }
                return Err(VulkanDrmImageError::DmabufExport(error));
            }
        };
        let dmabuf = unsafe { OwnedFd::from_raw_fd(raw_fd) };

        let plane_aspects = [
            vk::ImageAspectFlags::MEMORY_PLANE_0_EXT,
            vk::ImageAspectFlags::MEMORY_PLANE_1_EXT,
            vk::ImageAspectFlags::MEMORY_PLANE_2_EXT,
            vk::ImageAspectFlags::MEMORY_PLANE_3_EXT,
        ];
        let mut planes = Vec::with_capacity(format.memory_plane_count as usize);
        for aspect in plane_aspects.iter().take(format.memory_plane_count as usize) {
            let subresource = vk::ImageSubresource::default().aspect_mask(*aspect);
            let layout = unsafe { raw.get_image_subresource_layout(image, subresource) };
            let plane_end = layout.offset.checked_add(layout.size);
            if layout.size == 0
                || layout.row_pitch == 0
                || plane_end.is_none_or(|end| end > requirements.size)
            {
                unsafe {
                    raw.destroy_image(image, None);
                    raw.free_memory(memory, None);
                }
                return Err(VulkanDrmImageError::InvalidPlaneLayout);
            }

            planes.push(VulkanDrmPlaneLayout {
                offset: layout.offset,
                stride: layout.row_pitch,
                size: layout.size,
            });
        }

        Ok(VulkanDrmImage {
            device: self.device,
            image,
            memory,
            allocation_size: requirements.size,
            dmabuf,
            width,
            height,
            format: vk_format,
            drm_format: format,
            usage,
            planes,
        })
    }
}

/// Owns the modifier image, its dedicated exportable memory and DMA-BUF FD.
pub struct VulkanDrmImage<'device> {
    device: &'device VulkanLogicalDevice,
    image: vk::Image,
    memory: vk::DeviceMemory,
    allocation_size: vk::DeviceSize,
    dmabuf: OwnedFd,
    width: u32,
    height: u32,
    format: vk::Format,
    drm_format: DrmVulkanFormat,
    usage: vk::ImageUsageFlags,
    planes: Vec<VulkanDrmPlaneLayout>,
}

impl<'device> VulkanDrmImage<'device> {
    pub fn image(&self) -> vk::Image { self.image }
    pub fn usage(&self) -> vk::ImageUsageFlags { self.usage }
    pub(super) fn device(&self) -> &'device VulkanLogicalDevice { self.device }
    pub fn dmabuf(&self) -> &OwnedFd { &self.dmabuf }
    pub fn allocation_size(&self) -> vk::DeviceSize { self.allocation_size }
    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    pub fn format(&self) -> vk::Format { self.format }
    pub fn drm_format(&self) -> DrmVulkanFormat { self.drm_format }
    pub fn planes(&self) -> &[VulkanDrmPlaneLayout] { &self.planes }
}

impl Drop for VulkanDrmImage<'_> {
    fn drop(&mut self) {
        let raw = self.device.handle();
        unsafe {
            raw.destroy_image(self.image, None);
            raw.free_memory(self.memory, None);
        }
    }
}
