// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use aurea_compositor::drm::{
    frame_coordinator::{DrmOutputFrameCoordinator, DrmOutputFrameError},
    presentation::DrmFramebufferId,
};

fn fb(id: u64) -> DrmFramebufferId {
    DrmFramebufferId::new(id)
}

fn submit_for_render(coordinator: &mut DrmOutputFrameCoordinator, framebuffer: DrmFramebufferId) {
    coordinator.begin_render(framebuffer).unwrap();
    coordinator.mark_submitted(framebuffer).unwrap();
}

fn commit_frame(
    coordinator: &mut DrmOutputFrameCoordinator,
    framebuffer: DrmFramebufferId,
    sync_file_attached: bool,
) {
    coordinator.begin_presentation(framebuffer).unwrap();
    coordinator.presentation_test_passed().unwrap();
    coordinator
        .presentation_commit_accepted(sync_file_attached)
        .unwrap();
}

#[test]
fn framebuffer_registration_is_unique_per_output() {
    let mut coordinator = DrmOutputFrameCoordinator::new();
    let id = fb(1);

    coordinator.register_framebuffer(id).unwrap();

    assert_eq!(
        coordinator.register_framebuffer(id),
        Err(DrmOutputFrameError::DuplicateFramebuffer(id))
    );
    assert_eq!(coordinator.len(), 1);
}

#[test]
fn presentation_requires_a_submitted_gpu_frame() {
    let mut coordinator = DrmOutputFrameCoordinator::new();
    let id = fb(2);
    coordinator.register_framebuffer(id).unwrap();
    coordinator.begin_render(id).unwrap();

    assert_eq!(
        coordinator.begin_presentation(id),
        Err(DrmOutputFrameError::ResourceNotSubmitted(id))
    );
}

#[test]
fn first_page_flip_establishes_current_framebuffer() {
    let mut coordinator = DrmOutputFrameCoordinator::new();
    let id = fb(3);
    coordinator.register_framebuffer(id).unwrap();
    submit_for_render(&mut coordinator, id);
    coordinator.mark_gpu_complete(id).unwrap();

    commit_frame(&mut coordinator, id, false);
    assert_eq!(coordinator.pending(), Some(id));
    assert_eq!(coordinator.page_flip_complete(id), Ok(None));
    assert_eq!(coordinator.current(), Some(id));
    assert!(!coordinator.is_available(id).unwrap());
}

#[test]
fn previous_frame_is_released_only_after_the_next_page_flip() {
    let mut coordinator = DrmOutputFrameCoordinator::new();
    let first = fb(10);
    let second = fb(11);
    coordinator.register_framebuffer(first).unwrap();
    coordinator.register_framebuffer(second).unwrap();

    submit_for_render(&mut coordinator, first);
    coordinator.mark_gpu_complete(first).unwrap();
    commit_frame(&mut coordinator, first, false);
    coordinator.page_flip_complete(first).unwrap();

    submit_for_render(&mut coordinator, second);
    coordinator.mark_gpu_complete(second).unwrap();
    commit_frame(&mut coordinator, second, false);

    assert!(!coordinator.is_available(first).unwrap());
    assert_eq!(coordinator.page_flip_complete(second), Ok(Some(first)));
    assert!(coordinator.is_available(first).unwrap());
    assert!(!coordinator.is_available(second).unwrap());
}

#[test]
fn sync_file_handoff_does_not_require_gpu_completion_before_commit() {
    let mut coordinator = DrmOutputFrameCoordinator::new();
    let id = fb(20);
    coordinator.register_framebuffer(id).unwrap();
    submit_for_render(&mut coordinator, id);

    commit_frame(&mut coordinator, id, true);
    assert!(!coordinator.is_available(id).unwrap());

    coordinator.mark_gpu_complete(id).unwrap();
    assert!(!coordinator.is_available(id).unwrap());
    coordinator.page_flip_complete(id).unwrap();
    assert!(!coordinator.is_available(id).unwrap());
}

#[test]
fn failed_atomic_test_does_not_release_gpu_work_early() {
    let mut coordinator = DrmOutputFrameCoordinator::new();
    let id = fb(30);
    coordinator.register_framebuffer(id).unwrap();
    submit_for_render(&mut coordinator, id);

    coordinator.begin_presentation(id).unwrap();
    assert_eq!(coordinator.presentation_test_failed(), Ok(id));
    assert!(!coordinator.is_available(id).unwrap());

    coordinator.mark_gpu_complete(id).unwrap();
    assert!(coordinator.is_available(id).unwrap());
}

#[test]
fn failed_commit_keeps_previous_scanout_and_releases_candidate_after_gpu_completion() {
    let mut coordinator = DrmOutputFrameCoordinator::new();
    let current = fb(40);
    let candidate = fb(41);
    coordinator.register_framebuffer(current).unwrap();
    coordinator.register_framebuffer(candidate).unwrap();

    submit_for_render(&mut coordinator, current);
    coordinator.mark_gpu_complete(current).unwrap();
    commit_frame(&mut coordinator, current, false);
    coordinator.page_flip_complete(current).unwrap();

    submit_for_render(&mut coordinator, candidate);
    coordinator.begin_presentation(candidate).unwrap();
    coordinator.presentation_test_passed().unwrap();
    assert_eq!(coordinator.presentation_commit_failed(), Ok(candidate));

    assert_eq!(coordinator.current(), Some(current));
    assert!(!coordinator.is_available(candidate).unwrap());
    coordinator.mark_gpu_complete(candidate).unwrap();
    assert!(coordinator.is_available(candidate).unwrap());
}

#[test]
fn active_or_pending_framebuffer_cannot_be_unregistered() {
    let mut coordinator = DrmOutputFrameCoordinator::new();
    let id = fb(50);
    coordinator.register_framebuffer(id).unwrap();
    submit_for_render(&mut coordinator, id);
    coordinator.mark_gpu_complete(id).unwrap();
    commit_frame(&mut coordinator, id, false);

    assert_eq!(
        coordinator.unregister_framebuffer(id),
        Err(DrmOutputFrameError::FramebufferStillScannedOut(id))
    );

    coordinator.page_flip_complete(id).unwrap();
    assert_eq!(
        coordinator.unregister_framebuffer(id),
        Err(DrmOutputFrameError::FramebufferStillScannedOut(id))
    );
}

#[test]
fn framebuffer_can_be_unregistered_after_kms_and_gpu_release() {
    let mut coordinator = DrmOutputFrameCoordinator::new();
    let first = fb(60);
    let second = fb(61);
    coordinator.register_framebuffer(first).unwrap();
    coordinator.register_framebuffer(second).unwrap();

    submit_for_render(&mut coordinator, first);
    coordinator.mark_gpu_complete(first).unwrap();
    commit_frame(&mut coordinator, first, false);
    coordinator.page_flip_complete(first).unwrap();

    submit_for_render(&mut coordinator, second);
    coordinator.mark_gpu_complete(second).unwrap();
    commit_frame(&mut coordinator, second, false);
    coordinator.page_flip_complete(second).unwrap();

    coordinator.unregister_framebuffer(first).unwrap();
    assert_eq!(coordinator.len(), 1);
}

#[test]
fn unknown_framebuffer_is_rejected() {
    let coordinator = DrmOutputFrameCoordinator::new();

    assert_eq!(
        coordinator.resource_for(fb(999)),
        Err(DrmOutputFrameError::UnknownFramebuffer(fb(999)))
    );
}

#[test]
fn unfenced_kms_commit_is_rejected_until_gpu_completion() {
    let mut coordinator = DrmOutputFrameCoordinator::new();
    let id = fb(70);
    coordinator.register_framebuffer(id).unwrap();
    submit_for_render(&mut coordinator, id);

    coordinator.begin_presentation(id).unwrap();
    coordinator.presentation_test_passed().unwrap();

    assert_eq!(
        coordinator.validate_commit_ready(false),
        Err(DrmOutputFrameError::GpuWorkIncomplete(id))
    );

    coordinator.mark_gpu_complete(id).unwrap();
    assert_eq!(coordinator.validate_commit_ready(false), Ok(id));
    coordinator.presentation_commit_accepted(false).unwrap();
    coordinator.page_flip_complete(id).unwrap();
}

#[test]
fn failed_presentation_cannot_be_retried_with_the_same_framebuffer() {
    let mut coordinator = DrmOutputFrameCoordinator::new();
    let id = fb(71);
    coordinator.register_framebuffer(id).unwrap();
    submit_for_render(&mut coordinator, id);

    coordinator.begin_presentation(id).unwrap();
    coordinator.presentation_test_failed().unwrap();

    assert_eq!(
        coordinator.begin_presentation(id),
        Err(DrmOutputFrameError::ResourceAlreadyResolved(id))
    );
}

#[test]
fn foreign_owned_frame_requires_completed_acquire_before_reuse() {
    let mut coordinator = DrmOutputFrameCoordinator::new();
    let first = fb(90);
    let second = fb(91);
    coordinator.register_framebuffer(first).unwrap();
    coordinator.register_framebuffer(second).unwrap();

    submit_for_render(&mut coordinator, first);
    coordinator.mark_gpu_complete(first).unwrap();
    coordinator.mark_foreign_release_submitted(first).unwrap();
    commit_frame(&mut coordinator, first, false);
    assert_eq!(coordinator.page_flip_complete(first), Ok(None));

    submit_for_render(&mut coordinator, second);
    coordinator.mark_gpu_complete(second).unwrap();
    coordinator.mark_foreign_release_submitted(second).unwrap();
    commit_frame(&mut coordinator, second, false);

    assert_eq!(coordinator.page_flip_complete(second), Ok(Some(first)));
    assert!(!coordinator.is_available(first).unwrap());

    coordinator.mark_foreign_acquire_submitted(first).unwrap();
    assert!(!coordinator.is_available(first).unwrap());

    coordinator.mark_foreign_acquired(first).unwrap();
    assert!(coordinator.is_available(first).unwrap());
}

#[test]
fn separate_outputs_keep_independent_frame_lifecycles() {
    let mut left_output = DrmOutputFrameCoordinator::new();
    let mut right_output = DrmOutputFrameCoordinator::new();
    let left = fb(80);
    let right = fb(80);

    left_output.register_framebuffer(left).unwrap();
    right_output.register_framebuffer(right).unwrap();

    submit_for_render(&mut left_output, left);
    left_output.mark_gpu_complete(left).unwrap();
    commit_frame(&mut left_output, left, false);
    left_output.page_flip_complete(left).unwrap();

    assert_eq!(left_output.current(), Some(left));
    assert_eq!(right_output.current(), None);
    assert!(right_output.is_available(right).unwrap());
}
