// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Device-extension negotiation for AureaOS-owned Vulkan resources.

use std::ffi::CStr;

use smithay::backend::vulkan::PhysicalDevice;

/// Base extensions required for Vulkan DMA-BUF import/export.
const REQUIRED_DMABUF_EXTENSIONS: [&CStr; 2] = [
    c"VK_KHR_external_memory_fd",
    c"VK_EXT_external_memory_dma_buf",
];

/// Optional extension for DRM modifier image tiling.
const DRM_MODIFIER_EXTENSION: &CStr = c"VK_EXT_image_drm_format_modifier";

/// Optional extension for exporting Vulkan semaphore payloads as sync files.
const SYNC_FD_EXTENSION: &CStr = c"VK_KHR_external_semaphore_fd";

/// Optional extension for transferring DMA-BUF image ownership to foreign APIs.
const FOREIGN_QUEUE_FAMILY_EXTENSION: &CStr = c"VK_EXT_queue_family_foreign";

/// Negotiated extension set for one physical Vulkan device.
/// The plan must be used only to create a logical device for that same GPU.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VulkanDeviceExtensionPlan {
    enabled: Vec<&'static CStr>,
    missing: Vec<String>,
    supports_drm_format_modifier: bool,
    supports_sync_fd: bool,
    supports_foreign_queue_family: bool,
}

impl VulkanDeviceExtensionPlan {
    /// Inspects DMA-BUF and optional presentation extensions on a physical device.
    pub fn from_device(device: &PhysicalDevice) -> Self {
        let available = REQUIRED_DMABUF_EXTENSIONS
            .iter()
            .copied()
            .filter(|extension| device.has_device_extension(extension))
            .collect::<Vec<_>>();
        let supports_drm_format_modifier = device.has_device_extension(DRM_MODIFIER_EXTENSION);
        let supports_sync_fd = device.has_device_extension(SYNC_FD_EXTENSION);
        let supports_foreign_queue_family =
            device.has_device_extension(FOREIGN_QUEUE_FAMILY_EXTENSION);

        Self::from_available_extensions(
            available,
            supports_drm_format_modifier,
            supports_sync_fd,
            supports_foreign_queue_family,
        )
    }

    /// Builds a deterministic plan from extension names for policy tests.
    pub fn from_available_names<I, S>(available: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let available = available
            .into_iter()
            .map(|name| name.as_ref().to_owned())
            .collect::<std::collections::HashSet<_>>();

        let supports_drm_format_modifier =
            available.contains(DRM_MODIFIER_EXTENSION.to_str().unwrap_or_default());
        let supports_sync_fd = available.contains(SYNC_FD_EXTENSION.to_str().unwrap_or_default());
        let supports_foreign_queue_family =
            available.contains(FOREIGN_QUEUE_FAMILY_EXTENSION.to_str().unwrap_or_default());

        let mut enabled = REQUIRED_DMABUF_EXTENSIONS
            .iter()
            .copied()
            .filter(|extension| available.contains(extension.to_str().unwrap_or_default()))
            .collect::<Vec<_>>();

        for (supported, extension) in [
            (supports_drm_format_modifier, DRM_MODIFIER_EXTENSION),
            (supports_sync_fd, SYNC_FD_EXTENSION),
            (
                supports_foreign_queue_family,
                FOREIGN_QUEUE_FAMILY_EXTENSION,
            ),
        ] {
            if supported {
                enabled.push(extension);
            }
        }

        let missing = REQUIRED_DMABUF_EXTENSIONS
            .iter()
            .filter(|extension| !available.contains(extension.to_str().unwrap_or_default()))
            .map(|extension| extension.to_string_lossy().into_owned())
            .collect();

        Self {
            enabled,
            missing,
            supports_drm_format_modifier,
            supports_sync_fd,
            supports_foreign_queue_family,
        }
    }

    fn from_available_extensions(
        available: Vec<&'static CStr>,
        supports_drm_format_modifier: bool,
        supports_sync_fd: bool,
        supports_foreign_queue_family: bool,
    ) -> Self {
        let missing = REQUIRED_DMABUF_EXTENSIONS
            .iter()
            .filter(|extension| !available.contains(extension))
            .map(|extension| extension.to_string_lossy().into_owned())
            .collect();

        let mut enabled = available;
        for (supported, extension) in [
            (supports_drm_format_modifier, DRM_MODIFIER_EXTENSION),
            (supports_sync_fd, SYNC_FD_EXTENSION),
            (supports_foreign_queue_family, FOREIGN_QUEUE_FAMILY_EXTENSION),
        ] {
            if supported {
                enabled.push(extension);
            }
        }

        Self {
            enabled,
            missing,
            supports_drm_format_modifier,
            supports_sync_fd,
            supports_foreign_queue_family,
        }
    }

    /// Extension names to enable during logical-device creation.
    pub fn enabled_extensions(&self) -> &[&'static CStr] {
        &self.enabled
    }

    /// Base DMA-BUF extensions not exposed by the physical device.
    pub fn missing_extensions(&self) -> &[String] {
        &self.missing
    }

    /// Whether the base DMA-BUF import/export extension set is available.
    pub fn supports_dmabuf_interop(&self) -> bool {
        self.missing.is_empty()
    }

    /// Whether DRM modifier image tiling is available.
    pub fn supports_drm_format_modifier(&self) -> bool {
        self.supports_drm_format_modifier
    }

    /// Whether VK_KHR_external_semaphore_fd is available for sync-file export.
    /// Physical-device semaphore handle properties must still be checked.
    pub fn supports_sync_fd(&self) -> bool {
        self.supports_sync_fd
    }

    /// Whether VK_EXT_queue_family_foreign is available for external image ownership.
    pub fn supports_foreign_queue_family(&self) -> bool {
        self.supports_foreign_queue_family
    }

    /// Raw names for VkDeviceCreateInfo; pointers refer to static strings.
    pub(crate) fn enabled_name_pointers(&self) -> Vec<*const i8> {
        self.enabled
            .iter()
            .map(|extension| extension.as_ptr())
            .collect()
    }
}
