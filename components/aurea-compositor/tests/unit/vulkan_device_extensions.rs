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
    let plan = VulkanDeviceExtensionPlan::from_available_names(["VK_KHR_external_memory_fd"]);
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

#[test]
fn enables_sync_file_export_when_the_optional_extension_is_available() {
    let plan = VulkanDeviceExtensionPlan::from_available_names([
        REQUIRED[0],
        REQUIRED[1],
        REQUIRED[2],
        "VK_KHR_external_semaphore_fd",
    ]);

    assert!(plan.supports_dmabuf_interop());
    assert!(plan.supports_sync_fd());
    assert_eq!(plan.enabled_extensions().len(), REQUIRED.len() + 1);
    assert!(
        plan.enabled_extensions()
            .iter()
            .any(|name| { name.to_str() == Ok("VK_KHR_external_semaphore_fd") })
    );
}

#[test]
fn dma_buf_support_does_not_require_sync_file_export() {
    let plan = VulkanDeviceExtensionPlan::from_available_names(REQUIRED);

    assert!(plan.supports_dmabuf_interop());
    assert!(!plan.supports_sync_fd());
    assert_eq!(plan.enabled_extensions().len(), REQUIRED.len());
}

#[test]
fn enables_foreign_queue_family_support_when_the_optional_extension_is_available() {
    let plan = VulkanDeviceExtensionPlan::from_available_names([
        REQUIRED[0],
        REQUIRED[1],
        REQUIRED[2],
        "VK_EXT_queue_family_foreign",
    ]);

    assert!(plan.supports_dmabuf_interop());
    assert!(plan.supports_foreign_queue_family());
    assert_eq!(plan.enabled_extensions().len(), REQUIRED.len() + 1);
    assert!(
        plan.enabled_extensions()
            .iter()
            .any(|name| { name.to_str() == Ok("VK_EXT_queue_family_foreign") })
    );
}

#[test]
fn dma_buf_support_does_not_require_foreign_queue_family_support() {
    let plan = VulkanDeviceExtensionPlan::from_available_names(REQUIRED);

    assert!(plan.supports_dmabuf_interop());
    assert!(!plan.supports_foreign_queue_family());
    assert_eq!(plan.enabled_extensions().len(), REQUIRED.len());
}

#[test]
fn sync_file_and_foreign_queue_family_capabilities_are_independent() {
    let plan = VulkanDeviceExtensionPlan::from_available_names([
        REQUIRED[0],
        REQUIRED[1],
        REQUIRED[2],
        "VK_KHR_external_semaphore_fd",
    ]);

    assert!(plan.supports_sync_fd());
    assert!(!plan.supports_foreign_queue_family());
}
