
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Per-output frame request scheduling.

use std::collections::BTreeMap;

use crate::output::OutputId;

use super::token::{
    FrameLifecycleError,
    FrameRequestResult,
    FrameToken,
};

/// Internal scheduling state for one output.
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
/// Multiple requests received while a frame is being processed
/// are coalesced into one request for the next frame.
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
    pub fn request_frame(
        &mut self,
        output_id: OutputId,
    ) -> FrameRequestResult {
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
    pub fn begin_frame(
        &mut self,
        output_id: OutputId,
    ) -> Option<FrameToken> {
        let state = self.outputs.get_mut(&output_id)?;

        if !state.requested || state.in_flight {
            return None;
        }

        let sequence = state.next_sequence;
        state.next_sequence = state.next_sequence.wrapping_add(1);

        state.requested = false;
        state.in_flight = true;
        state.active_sequence = Some(sequence);

        Some(FrameToken::new(output_id, sequence))
    }

    /// Completes the frame identified by its token.
    pub fn complete_frame(
        &mut self,
        token: FrameToken,
    ) -> Result<(), FrameLifecycleError> {
        let state = self
            .outputs
            .get_mut(&token.output_id())
            .ok_or(
                FrameLifecycleError::NoFrameInFlight(token.output_id())
            )?;

        if !state.in_flight {
            return Err(
                FrameLifecycleError::NoFrameInFlight(token.output_id())
            );
        }

        if state.active_sequence != Some(token.sequence()) {
            return Err(
                FrameLifecycleError::StaleFrameToken(token.output_id())
            );
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
    pub fn remove_output(&mut self, output_id: OutputId) -> bool {
        self.outputs.remove(&output_id).is_some()
    }

    /// Returns whether an output has a pending frame.
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

    /// Returns the number of tracked outputs.
    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }
}
