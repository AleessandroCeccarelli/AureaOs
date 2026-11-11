// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use aurea_compositor::vulkan::{
    VulkanFrameResourceRegistry,
    VulkanFrameResourceRegistryError,
};

#[test]
fn registry_assigns_distinct_stable_identifiers() {
    let mut registry = VulkanFrameResourceRegistry::new();

    let first = registry.register().unwrap();
    let second = registry.register().unwrap();

    assert_ne!(first, second);
    assert_eq!(registry.len(), 2);
    assert!(registry.is_available(first).unwrap());
    assert!(registry.is_available(second).unwrap());
}

#[test]
fn resources_are_isolated_from_each_other() {
    let mut registry = VulkanFrameResourceRegistry::new();
    let first = registry.register().unwrap();
    let second = registry.register().unwrap();

    registry.begin_render(first).unwrap();
    registry.mark_submitted(first).unwrap();

    assert!(!registry.is_available(first).unwrap());
    assert!(registry.is_available(second).unwrap());
}

#[test]
fn unknown_resource_is_rejected() {
    let mut registry = VulkanFrameResourceRegistry::new();
    let id = registry.register().unwrap();
    registry.unregister(id).unwrap();

    assert_eq!(
        registry.begin_render(id),
        Err(VulkanFrameResourceRegistryError::UnknownResource(id))
    );
}

#[test]
fn busy_resource_cannot_be_unregistered() {
    let mut registry = VulkanFrameResourceRegistry::new();
    let id = registry.register().unwrap();

    registry.begin_render(id).unwrap();

    assert_eq!(
        registry.unregister(id),
        Err(VulkanFrameResourceRegistryError::ResourceBusy(id))
    );
    assert_eq!(registry.len(), 1);
}

#[test]
fn resource_can_be_unregistered_after_gpu_and_kms_release() {
    let mut registry = VulkanFrameResourceRegistry::new();
    let id = registry.register().unwrap();

    registry.begin_render(id).unwrap();
    registry.mark_submitted(id).unwrap();
    registry.mark_gpu_complete(id).unwrap();
    registry.mark_kms_committed(id).unwrap();

    assert_eq!(
        registry.unregister(id),
        Err(VulkanFrameResourceRegistryError::ResourceBusy(id))
    );

    registry.mark_kms_released(id).unwrap();
    registry.unregister(id).unwrap();

    assert!(registry.is_empty());
}

#[test]
fn sync_file_commit_keeps_resource_busy_until_both_owners_release_it() {
    let mut registry = VulkanFrameResourceRegistry::new();
    let id = registry.register().unwrap();

    registry.begin_render(id).unwrap();
    registry.mark_submitted(id).unwrap();
    registry.mark_kms_committed_with_sync_fd(id).unwrap();

    assert!(!registry.is_available(id).unwrap());

    registry.mark_gpu_complete(id).unwrap();
    assert!(!registry.is_available(id).unwrap());

    registry.mark_kms_released(id).unwrap();
    assert!(registry.is_available(id).unwrap());
}

#[test]
fn failed_presentation_keeps_resource_busy_until_gpu_completion() {
    let mut registry = VulkanFrameResourceRegistry::new();
    let id = registry.register().unwrap();

    registry.begin_render(id).unwrap();
    registry.mark_submitted(id).unwrap();
    registry.mark_presentation_skipped(id).unwrap();

    assert!(!registry.is_available(id).unwrap());

    registry.mark_gpu_complete(id).unwrap();
    assert!(registry.is_available(id).unwrap());
}

#[test]
fn foreign_queue_acquire_must_complete_before_resource_can_be_removed() {
    let mut registry = VulkanFrameResourceRegistry::new();
    let id = registry.register().unwrap();

    registry.begin_render(id).unwrap();
    registry.mark_submitted(id).unwrap();
    registry.mark_foreign_release_submitted(id).unwrap();
    registry.mark_kms_committed_with_sync_fd(id).unwrap();
    registry.mark_gpu_complete(id).unwrap();
    registry.mark_kms_released(id).unwrap();

    assert_eq!(
        registry.unregister(id),
        Err(VulkanFrameResourceRegistryError::ResourceBusy(id))
    );

    registry.mark_foreign_acquire_submitted(id).unwrap();
    assert_eq!(
        registry.unregister(id),
        Err(VulkanFrameResourceRegistryError::ResourceBusy(id))
    );

    registry.mark_foreign_acquired(id).unwrap();
    registry.unregister(id).unwrap();
    assert!(registry.is_empty());
}
