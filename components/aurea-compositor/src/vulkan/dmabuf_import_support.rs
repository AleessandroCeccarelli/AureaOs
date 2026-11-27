// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Device-specific validation for client DMA-BUF image imports.

use smithay::{
    backend::allocator::vulkan::format::get_vk_format,
    reexports::ash::vk,
};

use super::{DmabufImportLayout, VulkanLogicalDevice};

/// Exact Vulkan image configuration accepted for a sampled DMA-BUF import.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VulkanDmabufImportSupport {
    pub format: vk::Format,
    pub width: u32,
    pub height: u32,
    pub modifier: u64,
}

/// Why a client DMA-BUF cannot be imported by this Vulkan device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VulkanDmabufImportSupportError {
    MissingInteropExtensions,
    MissingModifierExtension,
    UnsupportedFormat,
    UnsupportedPlaneCount,
    InvalidDimensions,
    ImageConfigurationUnsupported,
    ExternalMemoryNotImportable,
}

/// A capability gate only: success does not import FDs or synchronize access.
impl VulkanDmabufImportSupport {
    pub(crate) fn query(
        device: &VulkanLogicalDevice,
        layout: &DmabufImportLayout,
    ) -> Result<Self, VulkanDmabufImportSupportError> {
        let extensions = device.extension_plan();
        if !extensions.supports_dmabuf_interop() {
            return Err(VulkanDmabufImportSupportError::MissingInteropExtensions);
        }
        if !extensions.supports_drm_format_modifier() {
            return Err(VulkanDmabufImportSupportError::MissingModifierExtension);
        }
        if layout.width() == 0 || layout.height() == 0 {
            return Err(VulkanDmabufImportSupportError::InvalidDimensions);
        }
        // Multi-plane and multi-FD binding is intentionally rejected until
        // independent plane memory allocation and binding are implemented.
        if layout.planes().len() != 1 {
            return Err(VulkanDmabufImportSupportError::UnsupportedPlaneCount);
        }

        let format = get_vk_format(layout.format().code)
            .ok_or(VulkanDmabufImportSupportError::UnsupportedFormat)?;
        let modifier = u64::from(layout.format().modifier);
        let mut modifier_query =
            vk::PhysicalDeviceImageDrmFormatModifierInfoEXT::default().drm_format_modifier(modifier);
        let mut external_query = vk::PhysicalDeviceExternalImageFormatInfo::default()
            .handle_type(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
        let image_query = vk::PhysicalDeviceImageFormatInfo2::default()
            .format(format)
            .ty(vk::ImageType::TYPE_2D)
            .tiling(vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT)
            .usage(vk::ImageUsageFlags::SAMPLED)
            .push_next(&mut modifier_query)
            .push_next(&mut external_query);

        let mut external_properties = vk::ExternalImageFormatProperties::default();
        let mut image_properties =
            vk::ImageFormatProperties2::default().push_next(&mut external_properties);
        unsafe {
            device.instance().get_physical_device_image_format_properties2(
                device.physical_device(),
                &image_query,
                &mut image_properties,
            )
        }
        .map_err(|_| VulkanDmabufImportSupportError::ImageConfigurationUnsupported)?;

        let external = external_properties.external_memory_properties;
        if !external.external_memory_features.contains(vk::ExternalMemoryFeatureFlags::IMPORTABLE)
            || !external.compatible_handle_types.contains(
                vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT,
            )
        {
            return Err(VulkanDmabufImportSupportError::ExternalMemoryNotImportable);
        }

        let limits = image_properties.image_format_properties;
        if layout.width() > limits.max_extent.width || layout.height() > limits.max_extent.height {
            return Err(VulkanDmabufImportSupportError::ImageConfigurationUnsupported);
        }

        Ok(Self { format, width: layout.width(), height: layout.height(), modifier })
    }
}
