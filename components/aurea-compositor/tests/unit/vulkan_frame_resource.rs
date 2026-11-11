// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use aurea_compositor::vulkan::{
    VulkanFrameResource,
    VulkanFrameResourceError,
};

#[test]
fn resource_is_available_on_creation() {
    assert!(VulkanFrameResource::new().is_available());
}

#[test]
fn resource_cannot_be_acquired_twice() {
    let mut resource = VulkanFrameResource::new();
    resource.begin_render().unwrap();

    assert_eq!(
        resource.begin_render(),
        Err(VulkanFrameResourceError::AlreadyInUse)
    );
}

#[test]
fn resource_waits_for_gpu_and_presentation_resolution() {
    let mut resource = VulkanFrameResource::new();
    resource.begin_render().unwrap();
    resource.mark_submitted().unwrap();
    resource.mark_gpu_complete().unwrap();

    assert!(!resource.is_available());

    resource.mark_presentation_skipped().unwrap();
    assert!(resource.is_available());
}

#[test]
fn resource_waits_for_kms_release_after_gpu_completion() {
    let mut resource = VulkanFrameResource::new();
    resource.begin_render().unwrap();
    resource.mark_submitted().unwrap();
    resource.mark_kms_committed().unwrap();
    resource.mark_gpu_complete().unwrap();

    assert!(!resource.is_available());

    resource.mark_kms_released().unwrap();
    assert!(resource.is_available());
}

#[test]
fn resource_waits_for_gpu_completion_after_kms_release() {
    let mut resource = VulkanFrameResource::new();
    resource.begin_render().unwrap();
    resource.mark_submitted().unwrap();
    resource.mark_kms_committed().unwrap();
    resource.mark_kms_released().unwrap();

    assert!(!resource.is_available());

    resource.mark_gpu_complete().unwrap();
    assert!(resource.is_available());
}

#[test]
fn failed_presentation_does_not_release_gpu_work_early() {
    let mut resource = VulkanFrameResource::new();
    resource.begin_render().unwrap();
    resource.mark_submitted().unwrap();
    resource.mark_presentation_skipped().unwrap();

    assert!(!resource.is_available());

    resource.mark_gpu_complete().unwrap();
    assert!(resource.is_available());
}

#[test]
fn kms_cannot_release_a_resource_it_does_not_own() {
    let mut resource = VulkanFrameResource::new();
    resource.begin_render().unwrap();
    resource.mark_submitted().unwrap();

    assert_eq!(
        resource.mark_kms_released(),
        Err(VulkanFrameResourceError::KmsDoesNotOwnResource)
    );
}

#[test]
fn presentation_cannot_be_resolved_twice() {
    let mut resource = VulkanFrameResource::new();
    resource.begin_render().unwrap();
    resource.mark_submitted().unwrap();
    resource.mark_presentation_skipped().unwrap();

    assert_eq!(
        resource.mark_presentation_skipped(),
        Err(VulkanFrameResourceError::PresentationAlreadyResolved)
    );
}

#[test]
fn kms_cannot_commit_a_resource_twice() {
    let mut resource = VulkanFrameResource::new();
    resource.begin_render().unwrap();
    resource.mark_submitted().unwrap();
    resource.mark_kms_committed().unwrap();

    assert_eq!(
        resource.mark_kms_committed(),
        Err(VulkanFrameResourceError::PresentationAlreadyResolved)
    );
}

#[test]
fn gpu_completion_before_submission_is_rejected() {
    let mut resource = VulkanFrameResource::new();

    assert_eq!(
        resource.mark_gpu_complete(),
        Err(VulkanFrameResourceError::NotSubmitted)
    );
}
