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
    resource.mark_gpu_complete().unwrap();
    resource.mark_kms_committed().unwrap();

    assert!(!resource.is_available());

    resource.mark_kms_released().unwrap();
    assert!(resource.is_available());
}

#[test]
fn kms_cannot_take_ownership_before_gpu_completion() {
    let mut resource = VulkanFrameResource::new();
    resource.begin_render().unwrap();
    resource.mark_submitted().unwrap();

    assert_eq!(
        resource.mark_kms_committed(),
        Err(VulkanFrameResourceError::GpuWorkIncomplete)
    );
    assert!(!resource.is_available());

    resource.mark_gpu_complete().unwrap();
    resource.mark_kms_committed().unwrap();
    resource.mark_kms_released().unwrap();
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
    resource.mark_gpu_complete().unwrap();
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


#[test]
fn submission_before_rendering_is_rejected() {
    let mut resource = VulkanFrameResource::new();

    assert_eq!(
        resource.mark_submitted(),
        Err(VulkanFrameResourceError::NotRendering)
    );
    assert!(resource.is_available());
}

#[test]
fn kms_cannot_commit_before_gpu_submission() {
    let mut resource = VulkanFrameResource::new();

    assert_eq!(
        resource.mark_kms_committed(),
        Err(VulkanFrameResourceError::NotSubmitted)
    );
    assert!(resource.is_available());
}

#[test]
fn kms_cannot_release_after_presentation_was_skipped() {
    let mut resource = VulkanFrameResource::new();
    resource.begin_render().unwrap();
    resource.mark_submitted().unwrap();
    resource.mark_presentation_skipped().unwrap();

    assert_eq!(
        resource.mark_kms_released(),
        Err(VulkanFrameResourceError::KmsDoesNotOwnResource)
    );
}

#[test]
fn presentation_cannot_be_skipped_after_kms_has_taken_ownership() {
    let mut resource = VulkanFrameResource::new();
    resource.begin_render().unwrap();
    resource.mark_submitted().unwrap();
    resource.mark_gpu_complete().unwrap();
    resource.mark_kms_committed().unwrap();

    assert_eq!(
        resource.mark_presentation_skipped(),
        Err(VulkanFrameResourceError::KmsAlreadyOwnsResource)
    );
    assert!(!resource.is_available());
}

#[test]
fn fully_released_resource_can_be_reused_for_a_second_frame() {
    let mut resource = VulkanFrameResource::new();

    resource.begin_render().unwrap();
    resource.mark_submitted().unwrap();
    resource.mark_gpu_complete().unwrap();
    resource.mark_kms_committed().unwrap();
    resource.mark_kms_released().unwrap();
    assert!(resource.is_available());

    resource.begin_render().unwrap();
    resource.mark_submitted().unwrap();
    resource.mark_presentation_skipped().unwrap();
    resource.mark_gpu_complete().unwrap();
    assert!(resource.is_available());
}

#[test]
fn sync_file_handoff_allows_kms_to_own_frame_before_gpu_completion() {
    let mut resource = VulkanFrameResource::new();
    resource.begin_render().unwrap();
    resource.mark_submitted().unwrap();

    resource.mark_kms_committed_with_sync_fd().unwrap();
    assert!(!resource.is_available());

    resource.mark_gpu_complete().unwrap();
    assert!(!resource.is_available());

    resource.mark_kms_released().unwrap();
    assert!(resource.is_available());
}

#[test]
fn sync_file_handoff_still_requires_gpu_and_kms_release() {
    let mut resource = VulkanFrameResource::new();
    resource.begin_render().unwrap();
    resource.mark_submitted().unwrap();
    resource.mark_kms_committed_with_sync_fd().unwrap();
    resource.mark_kms_released().unwrap();

    assert!(!resource.is_available());
    resource.mark_gpu_complete().unwrap();
    assert!(resource.is_available());
}

#[test]
fn foreign_owned_resource_cannot_be_reused_until_acquire_completes() {
    let mut resource = VulkanFrameResource::new();
    resource.begin_render().unwrap();
    resource.mark_submitted().unwrap();
    resource.mark_foreign_release_submitted().unwrap();
    resource.mark_kms_committed_with_sync_fd().unwrap();
    resource.mark_gpu_complete().unwrap();
    resource.mark_kms_released().unwrap();

    assert!(!resource.is_available());
    assert_eq!(
        resource.begin_render(),
        Err(VulkanFrameResourceError::AlreadyInUse)
    );

    assert_eq!(resource.mark_foreign_acquired(), Err(VulkanFrameResourceError::ForeignAcquireNotSubmitted));
    resource.mark_foreign_acquire_submitted().unwrap();
    assert!(!resource.is_available());
    resource.mark_foreign_acquired().unwrap();
    assert!(resource.is_available());
}

#[test]
fn foreign_resource_cannot_be_reused_after_acquire_submission_until_completion() {
    let mut resource = VulkanFrameResource::new();
    resource.begin_render().unwrap();
    resource.mark_submitted().unwrap();
    resource.mark_foreign_release_submitted().unwrap();
    resource.mark_kms_committed_with_sync_fd().unwrap();
    resource.mark_gpu_complete().unwrap();
    resource.mark_kms_released().unwrap();
    resource.mark_foreign_acquire_submitted().unwrap();

    assert!(!resource.is_available());
    assert_eq!(
        resource.mark_foreign_acquire_submitted(),
        Err(VulkanFrameResourceError::ForeignAcquireAlreadySubmitted)
    );
    assert_eq!(
        resource.begin_render(),
        Err(VulkanFrameResourceError::AlreadyInUse)
    );

    resource.mark_foreign_acquired().unwrap();
    assert!(resource.is_available());
}

#[test]
fn foreign_acquire_completion_is_rejected_without_acquire_submission() {
    let mut resource = VulkanFrameResource::new();
    resource.begin_render().unwrap();
    resource.mark_submitted().unwrap();
    resource.mark_foreign_release_submitted().unwrap();
    resource.mark_kms_committed_with_sync_fd().unwrap();
    resource.mark_gpu_complete().unwrap();
    resource.mark_kms_released().unwrap();

    assert_eq!(
        resource.mark_foreign_acquired(),
        Err(VulkanFrameResourceError::ForeignAcquireNotSubmitted)
    );
    assert!(!resource.is_available());
}

#[test]
fn foreign_acquire_is_rejected_before_foreign_release() {
    let mut resource = VulkanFrameResource::new();
    resource.begin_render().unwrap();
    resource.mark_submitted().unwrap();

    assert_eq!(
        resource.mark_foreign_acquired(),
        Err(VulkanFrameResourceError::ForeignOwnershipNotReleased)
    );
}

#[test]
fn foreign_release_cannot_be_recorded_twice_without_acquire() {
    let mut resource = VulkanFrameResource::new();
    resource.begin_render().unwrap();
    resource.mark_submitted().unwrap();
    resource.mark_foreign_release_submitted().unwrap();

    assert_eq!(
        resource.mark_foreign_release_submitted(),
        Err(VulkanFrameResourceError::ForeignOwnershipAlreadyReleased)
    );
}

#[test]
fn foreign_acquire_is_rejected_while_kms_still_owns_the_image() {
    let mut resource = VulkanFrameResource::new();
    resource.begin_render().unwrap();
    resource.mark_submitted().unwrap();
    resource.mark_foreign_release_submitted().unwrap();
    resource.mark_kms_committed_with_sync_fd().unwrap();

    assert_eq!(
        resource.mark_foreign_acquired(),
        Err(VulkanFrameResourceError::KmsAlreadyOwnsResource)
    );
}
