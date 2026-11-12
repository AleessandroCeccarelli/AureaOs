// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use aurea_compositor::drm::presentation::{
    DrmFramebufferId, DrmPresentationError, DrmPresentationLifecycle,
};
use aurea_compositor::drm::presentation_backend::{
    DrmPresentationBackend, DrmPresentationBackendError,
};

fn fb(value: u64) -> DrmFramebufferId {
    DrmFramebufferId::new(value)
}

#[test]
fn first_frame_becomes_current_after_page_flip() {
    let mut lifecycle = DrmPresentationLifecycle::new();
    lifecycle.begin_test(fb(1)).unwrap();
    lifecycle.test_passed().unwrap();
    lifecycle.commit_accepted().unwrap();

    assert_eq!(lifecycle.page_flip_complete(fb(1)), Ok(None));
    assert_eq!(lifecycle.current(), Some(fb(1)));
    assert!(!lifecycle.has_pending_transaction());
}

#[test]
fn previous_frame_is_released_only_after_page_flip() {
    let mut lifecycle = DrmPresentationLifecycle::new();
    lifecycle.begin_test(fb(1)).unwrap();
    lifecycle.test_passed().unwrap();
    lifecycle.commit_accepted().unwrap();
    lifecycle.page_flip_complete(fb(1)).unwrap();

    lifecycle.begin_test(fb(2)).unwrap();
    lifecycle.test_passed().unwrap();
    lifecycle.commit_accepted().unwrap();

    assert_eq!(lifecycle.current(), Some(fb(1)));
    assert_eq!(lifecycle.page_flip_complete(fb(2)), Ok(Some(fb(1))));
    assert_eq!(lifecycle.current(), Some(fb(2)));
}

#[test]
fn failed_atomic_test_returns_candidate_without_changing_current() {
    let mut lifecycle = DrmPresentationLifecycle::new();
    lifecycle.begin_test(fb(1)).unwrap();
    assert_eq!(lifecycle.test_failed(), Ok(fb(1)));
    assert_eq!(lifecycle.current(), None);
    assert!(!lifecycle.has_pending_transaction());
}

#[test]
fn failed_commit_preserves_current_framebuffer() {
    let mut lifecycle = DrmPresentationLifecycle::new();
    lifecycle.begin_test(fb(1)).unwrap();
    lifecycle.test_passed().unwrap();
    lifecycle.commit_accepted().unwrap();
    lifecycle.page_flip_complete(fb(1)).unwrap();

    lifecycle.begin_test(fb(2)).unwrap();
    lifecycle.test_passed().unwrap();
    assert_eq!(lifecycle.commit_failed(), Ok(fb(2)));
    assert_eq!(lifecycle.current(), Some(fb(1)));
}

#[test]
fn commit_cannot_be_accepted_before_atomic_test_passes() {
    let mut lifecycle = DrmPresentationLifecycle::new();
    lifecycle.begin_test(fb(1)).unwrap();

    assert_eq!(
        lifecycle.commit_accepted(),
        Err(DrmPresentationError::TestNotPassed)
    );
}

#[test]
fn only_one_presentation_transaction_may_be_pending() {
    let mut lifecycle = DrmPresentationLifecycle::new();
    lifecycle.begin_test(fb(1)).unwrap();

    assert_eq!(
        lifecycle.begin_test(fb(2)),
        Err(DrmPresentationError::TransactionInProgress)
    );
}

#[test]
fn page_flip_must_match_the_submitted_candidate() {
    let mut lifecycle = DrmPresentationLifecycle::new();
    lifecycle.begin_test(fb(1)).unwrap();
    lifecycle.test_passed().unwrap();
    lifecycle.commit_accepted().unwrap();

    assert_eq!(
        lifecycle.page_flip_complete(fb(2)),
        Err(DrmPresentationError::UnexpectedFramebuffer)
    );
    assert!(lifecycle.has_pending_transaction());
}

#[test]
fn current_framebuffer_cannot_be_submitted_again() {
    let mut lifecycle = DrmPresentationLifecycle::new();
    lifecycle.begin_test(fb(1)).unwrap();
    lifecycle.test_passed().unwrap();
    lifecycle.commit_accepted().unwrap();
    lifecycle.page_flip_complete(fb(1)).unwrap();

    assert_eq!(
        lifecycle.begin_test(fb(1)),
        Err(DrmPresentationError::CandidateIsCurrent)
    );
}

#[test]
fn test_passed_without_an_active_test_is_rejected() {
    let mut lifecycle = DrmPresentationLifecycle::new();

    assert_eq!(
        lifecycle.test_passed(),
        Err(DrmPresentationError::NoTransaction)
    );
}

#[test]
fn page_flip_cannot_complete_before_commit_acceptance() {
    let mut lifecycle = DrmPresentationLifecycle::new();
    lifecycle.begin_test(fb(1)).unwrap();
    lifecycle.test_passed().unwrap();

    assert_eq!(
        lifecycle.page_flip_complete(fb(1)),
        Err(DrmPresentationError::NoTransaction)
    );
    assert_eq!(lifecycle.current(), None);
    assert!(lifecycle.has_pending_transaction());
}

#[test]
fn failed_atomic_test_can_be_retried_with_the_same_candidate() {
    let mut lifecycle = DrmPresentationLifecycle::new();
    lifecycle.begin_test(fb(7)).unwrap();
    assert_eq!(lifecycle.test_failed(), Ok(fb(7)));

    lifecycle.begin_test(fb(7)).unwrap();
    lifecycle.test_passed().unwrap();
    lifecycle.commit_accepted().unwrap();
    assert_eq!(lifecycle.page_flip_complete(fb(7)), Ok(None));
    assert_eq!(lifecycle.current(), Some(fb(7)));
}

#[test]
fn commit_failure_before_test_pass_is_rejected_without_losing_transaction() {
    let mut lifecycle = DrmPresentationLifecycle::new();
    lifecycle.begin_test(fb(3)).unwrap();

    assert_eq!(
        lifecycle.commit_failed(),
        Err(DrmPresentationError::TestNotPassed)
    );
    assert!(lifecycle.has_pending_transaction());

    lifecycle.test_passed().unwrap();
    assert_eq!(lifecycle.commit_failed(), Ok(fb(3)));
    assert!(!lifecycle.has_pending_transaction());
}

#[test]
fn synchronized_scanout_accepts_planes_without_in_fences() {
    assert_eq!(
        DrmPresentationBackend::validate_plane_synchronization(&[false], false),
        Ok(())
    );
    assert_eq!(
        DrmPresentationBackend::validate_plane_synchronization(&[true, false], false),
        Ok(())
    );
}

#[test]
fn asynchronous_scanout_requires_an_in_fence_for_every_active_plane() {
    assert_eq!(
        DrmPresentationBackend::validate_plane_synchronization(&[true, true], true),
        Ok(())
    );
    assert_eq!(
        DrmPresentationBackend::validate_plane_synchronization(&[true, false], true),
        Err(DrmPresentationBackendError::MissingInFence)
    );
}

#[test]
fn scanout_rejects_an_empty_active_plane_set() {
    assert_eq!(
        DrmPresentationBackend::validate_plane_synchronization(&[], false),
        Err(DrmPresentationBackendError::MissingActivePlane)
    );
    assert_eq!(
        DrmPresentationBackend::validate_plane_synchronization(&[], true),
        Err(DrmPresentationBackendError::MissingActivePlane)
    );
}
