// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use aurea_compositor::drm::presentation::{
    DrmFramebufferId,
    DrmPresentationError,
    DrmPresentationLifecycle,
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
