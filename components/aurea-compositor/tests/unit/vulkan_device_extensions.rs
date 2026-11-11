// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Hardware-independent tests for Vulkan device-extension policy.

use aurea_compositor::vulkan::VulkanDeviceExtensionPlan;

const REQUIRED: [&str; 3] = [
    "VK_KHR_external_memory_fd",
    "VK_EXT_external_memory_dma_buf",
    "VK_EXT_image_drm_format_modifier",
];

#[test]
fn enables_all_required_dma_buf_extensions_when_available() {
    let plan = VulkanDeviceExtensionPlan::from_available_names(REQUIRED);
    assert!(plan.supports_dmabuf_interop());
    assert!(plan.missing_extensions().is_empty());
    assert_eq!(plan.enabled_extensions().len(), REQUIRED.len());
}

#[test]
fn reports_each_missing_extension() {
    let plan = VulkanDeviceExtensionPlan::from_available_names([
        "VK_KHR_external_memory_fd",
    ]);
    assert!(!plan.supports_dmabuf_interop());
    assert_eq!(plan.enabled_extensions().len(), 1);
    assert_eq!(plan.missing_extensions().len(), 2);
}

#[test]
fn ignores_unrelated_device_extensions() {
    let plan = VulkanDeviceExtensionPlan::from_available_names([
        "VK_KHR_external_memory_fd",
        "VK_EXT_external_memory_dma_buf",
        "VK_EXT_image_drm_format_modifier",
        "VK_EXT_unrelated_extension",
    ]);
    assert!(plan.supports_dmabuf_interop());
    assert_eq!(plan.enabled_extensions().len(), REQUIRED.len());
}
