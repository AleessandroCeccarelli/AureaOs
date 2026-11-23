// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Hardware-independent tests for Vulkan device-extension policy.

use aurea_compositor::vulkan::VulkanDeviceExtensionPlan;

const BASE: [&str; 2] = [
    "VK_KHR_external_memory_fd",
    "VK_EXT_external_memory_dma_buf",
];
const MODIFIER: &str = "VK_EXT_image_drm_format_modifier";

#[test]
fn enables_base_dma_buf_extensions_without_requiring_modifier_tiling() {
    let plan = VulkanDeviceExtensionPlan::from_available_names(BASE);

    assert!(plan.supports_dmabuf_interop());
    assert!(!plan.supports_drm_format_modifier());
    assert!(plan.missing_extensions().is_empty());
    assert_eq!(plan.enabled_extensions().len(), BASE.len());
}

#[test]
fn reports_each_missing_base_dma_buf_extension() {
    let plan = VulkanDeviceExtensionPlan::from_available_names(["VK_KHR_external_memory_fd"]);

    assert!(!plan.supports_dmabuf_interop());
    assert_eq!(plan.enabled_extensions().len(), 1);
    assert_eq!(plan.missing_extensions().len(), 1);
    assert_eq!(
        plan.missing_extensions()[0],
        "VK_EXT_external_memory_dma_buf"
    );
}

#[test]
fn enables_modifier_tiling_only_when_exposed() {
    let plan = VulkanDeviceExtensionPlan::from_available_names([BASE[0], BASE[1], MODIFIER]);

    assert!(plan.supports_dmabuf_interop());
    assert!(plan.supports_drm_format_modifier());
    assert_eq!(plan.enabled_extensions().len(), BASE.len() + 1);
}

#[test]
fn ignores_unrelated_device_extensions() {
    let plan = VulkanDeviceExtensionPlan::from_available_names([
        BASE[0],
        BASE[1],
        MODIFIER,
        "VK_EXT_unrelated_extension",
    ]);

    assert!(plan.supports_dmabuf_interop());
    assert!(plan.supports_drm_format_modifier());
    assert_eq!(plan.enabled_extensions().len(), BASE.len() + 1);
}

#[test]
fn enables_sync_file_export_when_the_optional_extension_is_available() {
    let plan = VulkanDeviceExtensionPlan::from_available_names([
        BASE[0],
        BASE[1],
        "VK_KHR_external_semaphore_fd",
    ]);

    assert!(plan.supports_dmabuf_interop());
    assert!(plan.supports_sync_fd());
    assert_eq!(plan.enabled_extensions().len(), BASE.len() + 1);
}

#[test]
fn enables_foreign_queue_family_support_independently() {
    let plan = VulkanDeviceExtensionPlan::from_available_names([
        BASE[0],
        BASE[1],
        "VK_EXT_queue_family_foreign",
    ]);

    assert!(plan.supports_dmabuf_interop());
    assert!(plan.supports_foreign_queue_family());
    assert_eq!(plan.enabled_extensions().len(), BASE.len() + 1);
}

#[test]
fn sync_file_and_foreign_queue_family_capabilities_are_independent() {
    let plan = VulkanDeviceExtensionPlan::from_available_names([
        BASE[0],
        BASE[1],
        "VK_KHR_external_semaphore_fd",
    ]);

    assert!(plan.supports_sync_fd());
    assert!(!plan.supports_foreign_queue_family());
}
