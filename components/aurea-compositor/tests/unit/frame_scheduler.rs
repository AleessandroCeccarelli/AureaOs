
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use aurea_compositor::frame::{
    FrameLifecycleError,
    FrameRequestResult,
    FrameScheduler,
};

use aurea_compositor::output::OutputId;

const OUTPUT_A: OutputId = 1;
const OUTPUT_B: OutputId = 2;

#[test]
fn new_scheduler_has_no_outputs() {
    let scheduler = FrameScheduler::new();

    assert_eq!(scheduler.output_count(), 0);
    assert!(!scheduler.has_pending_frame(OUTPUT_A));
    assert!(!scheduler.has_frame_in_flight(OUTPUT_A));
}

#[test]
fn first_request_is_queued() {
    let mut scheduler = FrameScheduler::new();

    assert_eq!(
        scheduler.request_frame(OUTPUT_A),
        FrameRequestResult::Queued
    );

    assert!(scheduler.has_pending_frame(OUTPUT_A));
}

#[test]
fn repeated_requests_are_coalesced() {
    let mut scheduler = FrameScheduler::new();

    assert_eq!(
        scheduler.request_frame(OUTPUT_A),
        FrameRequestResult::Queued
    );

    assert_eq!(
        scheduler.request_frame(OUTPUT_A),
        FrameRequestResult::AlreadyQueued
    );
}

#[test]
fn requested_frame_can_be_started() {
    let mut scheduler = FrameScheduler::new();

    scheduler.request_frame(OUTPUT_A);

    let token = scheduler.begin_frame(OUTPUT_A);

    assert!(token.is_some());
    assert!(!scheduler.has_pending_frame(OUTPUT_A));
    assert!(scheduler.has_frame_in_flight(OUTPUT_A));
}

#[test]
fn frame_cannot_start_without_request() {
    let mut scheduler = FrameScheduler::new();

    assert!(scheduler.begin_frame(OUTPUT_A).is_none());
}

#[test]
fn only_one_frame_can_be_in_flight_per_output() {
    let mut scheduler = FrameScheduler::new();

    scheduler.request_frame(OUTPUT_A);

    assert!(scheduler.begin_frame(OUTPUT_A).is_some());
    assert!(scheduler.begin_frame(OUTPUT_A).is_none());
}

#[test]
fn requests_during_frame_are_deferred() {
    let mut scheduler = FrameScheduler::new();

    scheduler.request_frame(OUTPUT_A);
    let token = scheduler.begin_frame(OUTPUT_A).unwrap();

    assert_eq!(
        scheduler.request_frame(OUTPUT_A),
        FrameRequestResult::DeferredUntilCompletion
    );

    assert!(scheduler.has_frame_in_flight(OUTPUT_A));
    assert!(!scheduler.has_pending_frame(OUTPUT_A));

    scheduler.complete_frame(token).unwrap();

    assert!(!scheduler.has_frame_in_flight(OUTPUT_A));
    assert!(scheduler.has_pending_frame(OUTPUT_A));
}

#[test]
fn multiple_requests_during_frame_are_coalesced() {
    let mut scheduler = FrameScheduler::new();

    scheduler.request_frame(OUTPUT_A);
    let token = scheduler.begin_frame(OUTPUT_A).unwrap();

    assert_eq!(
        scheduler.request_frame(OUTPUT_A),
        FrameRequestResult::DeferredUntilCompletion
    );

    assert_eq!(
        scheduler.request_frame(OUTPUT_A),
        FrameRequestResult::AlreadyQueued
    );

    scheduler.complete_frame(token).unwrap();

    assert!(scheduler.has_pending_frame(OUTPUT_A));
}

#[test]
fn outputs_are_scheduled_independently() {
    let mut scheduler = FrameScheduler::new();

    scheduler.request_frame(OUTPUT_A);
    scheduler.request_frame(OUTPUT_B);

    assert!(scheduler.begin_frame(OUTPUT_A).is_some());
    assert!(scheduler.begin_frame(OUTPUT_B).is_some());
}

#[test]
fn stale_frame_token_is_rejected() {
    let mut scheduler = FrameScheduler::new();

    scheduler.request_frame(OUTPUT_A);
    let first = scheduler.begin_frame(OUTPUT_A).unwrap();

    scheduler.complete_frame(first).unwrap();

    scheduler.request_frame(OUTPUT_A);
    let second = scheduler.begin_frame(OUTPUT_A).unwrap();

    assert_ne!(first.sequence(), second.sequence());

    assert_eq!(
        scheduler.complete_frame(first),
        Err(FrameLifecycleError::StaleFrameToken(OUTPUT_A))
    );

    assert!(scheduler.has_frame_in_flight(OUTPUT_A));
}

#[test]
fn completed_frame_can_be_followed_by_another_frame() {
    let mut scheduler = FrameScheduler::new();

    scheduler.request_frame(OUTPUT_A);
    let first = scheduler.begin_frame(OUTPUT_A).unwrap();

    scheduler.complete_frame(first).unwrap();

    scheduler.request_frame(OUTPUT_A);
    let second = scheduler.begin_frame(OUTPUT_A).unwrap();

    assert_ne!(first.sequence(), second.sequence());
}

#[test]
fn cancellation_removes_waiting_request() {
    let mut scheduler = FrameScheduler::new();

    scheduler.request_frame(OUTPUT_A);

    assert!(scheduler.cancel_request(OUTPUT_A));
    assert!(!scheduler.has_pending_frame(OUTPUT_A));
    assert!(!scheduler.cancel_request(OUTPUT_A));
}

#[test]
fn in_flight_frame_cannot_be_cancelled() {
    let mut scheduler = FrameScheduler::new();

    scheduler.request_frame(OUTPUT_A);
    let token = scheduler.begin_frame(OUTPUT_A).unwrap();

    assert!(!scheduler.cancel_request(OUTPUT_A));
    assert!(scheduler.has_frame_in_flight(OUTPUT_A));

    scheduler.complete_frame(token).unwrap();
}

#[test]
fn disconnected_output_state_can_be_removed() {
    let mut scheduler = FrameScheduler::new();

    scheduler.request_frame(OUTPUT_A);

    assert!(scheduler.remove_output(OUTPUT_A));
    assert_eq!(scheduler.output_count(), 0);
    assert!(!scheduler.remove_output(OUTPUT_A));
}

#[test]
fn completing_removed_output_returns_error() {
    let mut scheduler = FrameScheduler::new();

    scheduler.request_frame(OUTPUT_A);
    let token = scheduler.begin_frame(OUTPUT_A).unwrap();

    scheduler.remove_output(OUTPUT_A);

    assert_eq!(
        scheduler.complete_frame(token),
        Err(FrameLifecycleError::NoFrameInFlight(OUTPUT_A))
    );
}
