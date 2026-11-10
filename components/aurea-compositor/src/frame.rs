
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Per-output frame scheduling and lifecycle management.

use std::collections::BTreeMap;

use crate::output::OutputId;

/// Result of requesting a frame for an output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameRequestResult {
    /// A new frame request has been queued.
    Queued,

    /// A frame request was already waiting.
    AlreadyQueued,

    /// A frame is currently in flight. Another request will be
    /// queued automatically after the current frame completes.
    DeferredUntilCompletion,
}

/// Identifies one frame being processed by the compositor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameToken {
    output_id: OutputId,
    sequence: u64,
}

impl FrameToken {
    /// Returns the output associated with this frame.
    pub fn output_id(self) -> OutputId {
        self.output_id
    }

    /// Returns the monotonically increasing frame sequence.
    pub fn sequence(self) -> u64 {
        self.sequence
    }
}

/// Errors returned by frame lifecycle operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameLifecycleError {
    /// No frame is currently in flight for the specified output.
    NoFrameInFlight(OutputId),

    /// The token does not identify the currently active frame.
    StaleFrameToken(OutputId),
}

impl std::fmt::Display for FrameLifecycleError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoFrameInFlight(output_id) => {
                write!(formatter, "No frame is in flight for output {output_id}")
            }
            Self::StaleFrameToken(output_id) => {
                write!(formatter, "Frame token is stale for output {output_id}")
            }
        }
    }
}

impl std::error::Error for FrameLifecycleError {}

/// Internal lifecycle state for one output.
#[derive(Debug, Default)]
struct OutputFrameState {
    requested: bool,
    in_flight: bool,
    pending_after_completion: bool,
    next_sequence: u64,
    active_sequence: Option<u64>,
}

/// Coordinates frame requests independently for each display output.
///
/// Multiple requests received while a frame is being processed are
/// coalesced into one request for the next frame. This prevents an
/// unbounded queue of redundant rendering work.
#[derive(Debug, Default)]
pub struct FrameScheduler {
    outputs: BTreeMap<OutputId, OutputFrameState>,
}

impl FrameScheduler {
    /// Creates an empty frame scheduler.
    pub fn new() -> Self {
        Self::default()
    }

    /// Requests a frame for the specified output.
    ///
    /// If a frame is already in flight, the request is deferred until
    /// that frame completes. Repeated requests are coalesced.
    pub fn request_frame(&mut self, output_id: OutputId) -> FrameRequestResult {
        let state = self.outputs.entry(output_id).or_default();

        if state.in_flight {
            if state.pending_after_completion {
                return FrameRequestResult::AlreadyQueued;
            }

            state.pending_after_completion = true;
            return FrameRequestResult::DeferredUntilCompletion;
        }

        if state.requested {
            return FrameRequestResult::AlreadyQueued;
        }

        state.requested = true;
        FrameRequestResult::Queued
    }

    /// Starts a previously requested frame.
    ///
    /// Returns `None` when no request is waiting or another frame is
    /// already in flight for the output.
    pub fn begin_frame(&mut self, output_id: OutputId) -> Option<FrameToken> {
        let state = self.outputs.get_mut(&output_id)?;

        if !state.requested || state.in_flight {
            return None;
        }

        let sequence = state.next_sequence;
        state.next_sequence = state.next_sequence.wrapping_add(1);

        state.requested = false;
        state.in_flight = true;
        state.active_sequence = Some(sequence);

        Some(FrameToken {
            output_id,
            sequence,
        })
    }

    /// Completes the frame identified by the supplied token.
    ///
    /// If another frame was requested while this frame was in flight,
    /// that request becomes available through `begin_frame`.
    pub fn complete_frame(
        &mut self,
        token: FrameToken,
    ) -> Result<(), FrameLifecycleError> {
        let state = self
            .outputs
            .get_mut(&token.output_id)
            .ok_or(FrameLifecycleError::NoFrameInFlight(token.output_id))?;

        if !state.in_flight {
            return Err(FrameLifecycleError::NoFrameInFlight(token.output_id));
        }

        if state.active_sequence != Some(token.sequence) {
            return Err(FrameLifecycleError::StaleFrameToken(token.output_id));
        }

        state.in_flight = false;
        state.active_sequence = None;

        if state.pending_after_completion {
            state.pending_after_completion = false;
            state.requested = true;
        }

        Ok(())
    }

    /// Cancels a waiting frame request.
    ///
    /// An in-flight frame cannot be cancelled through this operation.
    pub fn cancel_request(&mut self, output_id: OutputId) -> bool {
        let Some(state) = self.outputs.get_mut(&output_id) else {
            return false;
        };

        if state.in_flight {
            return false;
        }

        let was_requested = state.requested;
        state.requested = false;

        was_requested
    }

    /// Removes all scheduling state for a disconnected output.
    ///
    /// Returns whether an output entry existed.
    pub fn remove_output(&mut self, output_id: OutputId) -> bool {
        self.outputs.remove(&output_id).is_some()
    }

    /// Returns whether an output has a frame waiting to begin.
    pub fn has_pending_frame(&self, output_id: OutputId) -> bool {
        self.outputs
            .get(&output_id)
            .is_some_and(|state| state.requested)
    }

    /// Returns whether an output currently has a frame in flight.
    pub fn has_frame_in_flight(&self, output_id: OutputId) -> bool {
        self.outputs
            .get(&output_id)
            .is_some_and(|state| state.in_flight)
    }

    /// Returns the number of outputs tracked by the scheduler.
    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let first = scheduler.begin_frame(OUTPUT_A);

        assert!(first.is_some());
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

        let frame_a = scheduler.begin_frame(OUTPUT_A);
        let frame_b = scheduler.begin_frame(OUTPUT_B);

        assert!(frame_a.is_some());
        assert!(frame_b.is_some());
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
    fn completing_unknown_output_returns_error() {
        let mut scheduler = FrameScheduler::new();

        let token = FrameToken {
            output_id: OUTPUT_A,
            sequence: 0,
        };

        assert_eq!(
            scheduler.complete_frame(token),
            Err(FrameLifecycleError::NoFrameInFlight(OUTPUT_A))
        );
    }
}
