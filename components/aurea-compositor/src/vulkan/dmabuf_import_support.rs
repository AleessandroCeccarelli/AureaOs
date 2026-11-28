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
    UnsupportedModifier,
    ModifierPlaneCountMismatch,
    UnsupportedPlaneCount,
    InvalidDimensions,
    ImageConfigurationUnsupported,
    ExternalMemoryNotImportable,
    MissingForeignQueueFamily,
    MissingSyncFdImport,
    DeviceUnusable,
}

/// A capability gate only: success does not import FDs or synchronize access.
impl VulkanDmabufImportSupport {
    /// Enumerates only single-plane DRM format/modifier pairs that this exact
    /// Vulkan device can sample from imported DMA-BUF memory.
    ///
    /// The returned set is suitable for linux-dmabuf v3 advertisement. It is
    /// intentionally narrower than the device's raw modifier list: Aurea
    /// requires sampled usage, external-memory importability and one memory
    /// plane because the current importer binds exactly one plane.
    pub(crate) fn supported_formats(
        device: &VulkanLogicalDevice,
    ) -> Vec<smithay::backend::allocator::Format> {
        use smithay::backend::allocator::{Format, Modifier};
        use smithay::backend::allocator::vulkan::format::{get_vk_format, known_formats};

        if device.is_unusable() {
            return Vec::new();
        }
        let extensions = device.extension_plan();
        if !extensions.supports_dmabuf_interop()
            || !extensions.supports_drm_format_modifier()
            || !device.supports_foreign_queue_family()
            || !device.supports_sync_fd_import()
        {
            return Vec::new();
        }

        let mut supported = Vec::new();
        for fourcc in known_formats().iter().copied() {
            let Some(format) = get_vk_format(fourcc) else { continue };
            let count = {
                let mut modifier_list = vk::DrmFormatModifierPropertiesListEXT::default();
                let mut format_properties =
                    vk::FormatProperties2::default().push_next(&mut modifier_list);
                unsafe {
                    device.instance().get_physical_device_format_properties2(
                        device.physical_device(),
                        format,
                        &mut format_properties,
                    );
                }
                modifier_list.drm_format_modifier_count as usize
            };
            if count == 0 { continue; }
            let mut properties = vec![vk::DrmFormatModifierPropertiesEXT::default(); count];
            {
                let mut modifier_list = vk::DrmFormatModifierPropertiesListEXT::default()
                    .drm_format_modifier_properties(&mut properties);
                let mut format_properties =
                    vk::FormatProperties2::default().push_next(&mut modifier_list);
                unsafe {
                    device.instance().get_physical_device_format_properties2(
                        device.physical_device(),
                        format,
                        &mut format_properties,
                    );
                }
            }

            for modifier in properties {
                if modifier.drm_format_modifier_plane_count != 1
                    || !modifier
                        .drm_format_modifier_tiling_features
                        .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE)
                {
                    continue;
                }

                let mut modifier_info =
                    vk::PhysicalDeviceImageDrmFormatModifierInfoEXT::default()
                        .drm_format_modifier(modifier.drm_format_modifier);
                let mut external_info =
                    vk::PhysicalDeviceExternalImageFormatInfo::default()
                        .handle_type(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
                let image_info = vk::PhysicalDeviceImageFormatInfo2::default()
                    .format(format)
                    .ty(vk::ImageType::TYPE_2D)
                    .tiling(vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT)
                    .usage(vk::ImageUsageFlags::SAMPLED)
                    .push_next(&mut modifier_info)
                    .push_next(&mut external_info);
                let mut external_properties = vk::ExternalImageFormatProperties::default();
                let mut image_properties =
                    vk::ImageFormatProperties2::default().push_next(&mut external_properties);
                if unsafe {
                    device.instance().get_physical_device_image_format_properties2(
                        device.physical_device(),
                        &image_info,
                        &mut image_properties,
                    )
                }.is_err() {
                    continue;
                }
                let external = external_properties.external_memory_properties;
                if !external.external_memory_features
                    .contains(vk::ExternalMemoryFeatureFlags::IMPORTABLE)
                    || !external.compatible_handle_types
                        .contains(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT)
                {
                    continue;
                }

                supported.push(Format {
                    code: fourcc,
                    modifier: Modifier::from(modifier.drm_format_modifier),
                });
            }
        }
        supported
    }

    pub(crate) fn query(
        device: &VulkanLogicalDevice,
        layout: &DmabufImportLayout,
    ) -> Result<Self, VulkanDmabufImportSupportError> {
        if device.is_unusable() {
            return Err(VulkanDmabufImportSupportError::DeviceUnusable);
        }
        if !device.supports_foreign_queue_family() {
            return Err(VulkanDmabufImportSupportError::MissingForeignQueueFamily);
        }
        if !device.supports_sync_fd_import() {
            return Err(VulkanDmabufImportSupportError::MissingSyncFdImport);
        }

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

        // The image-format query below validates the requested usage and
        // external-memory handle type. First verify that this exact modifier
        // is advertised for the mapped Vulkan format and that its Vulkan
        // memory-plane count agrees with the DMA-BUF metadata.
        let modifier_count = {
            let mut modifier_list = vk::DrmFormatModifierPropertiesListEXT::default();
            let mut format_properties =
                vk::FormatProperties2::default().push_next(&mut modifier_list);
            unsafe {
                device.instance().get_physical_device_format_properties2(
                    device.physical_device(),
                    format,
                    &mut format_properties,
                )
            };
            modifier_list.drm_format_modifier_count as usize
        };
        if modifier_count == 0 {
            return Err(VulkanDmabufImportSupportError::UnsupportedModifier);
        }
        let mut modifier_properties = vec![
            vk::DrmFormatModifierPropertiesEXT::default();
            modifier_count
        ];
        {
            let mut modifier_list = vk::DrmFormatModifierPropertiesListEXT::default()
                .drm_format_modifier_properties(&mut modifier_properties);
            let mut format_properties =
                vk::FormatProperties2::default().push_next(&mut modifier_list);
            unsafe {
                device.instance().get_physical_device_format_properties2(
                    device.physical_device(),
                    format,
                    &mut format_properties,
                )
            };
        }

        let modifier_properties = modifier_properties
            .iter()
            .find(|properties| properties.drm_format_modifier == modifier)
            .ok_or(VulkanDmabufImportSupportError::UnsupportedModifier)?;
        if modifier_properties.drm_format_modifier_plane_count as usize != layout.planes().len() {
            return Err(VulkanDmabufImportSupportError::ModifierPlaneCountMismatch);
        }
        if !modifier_properties
            .drm_format_modifier_tiling_features
            .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE)
        {
            return Err(VulkanDmabufImportSupportError::ImageConfigurationUnsupported);
        }

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
        let limits = {
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
            image_properties.image_format_properties
        };
        let external = external_properties.external_memory_properties;
        if !external.external_memory_features.contains(vk::ExternalMemoryFeatureFlags::IMPORTABLE)
            || !external.compatible_handle_types.contains(
                vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT,
            )
        {
            return Err(VulkanDmabufImportSupportError::ExternalMemoryNotImportable);
        }

        if layout.width() > limits.max_extent.width || layout.height() > limits.max_extent.height {
            return Err(VulkanDmabufImportSupportError::ImageConfigurationUnsupported);
        }

        Ok(Self { format, width: layout.width(), height: layout.height(), modifier })
    }
}
