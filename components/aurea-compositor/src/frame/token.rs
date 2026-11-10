
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Frame identifiers and lifecycle result types.

use crate::output::OutputId;

/// Result of requesting a frame for an output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameRequestResult {
    /// A new frame request has been queued.
    Queued,

    /// A frame request was already waiting.
    AlreadyQueued,

    /// A request was deferred until the current frame completes.
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

    /// Creates a frame token.
    pub(super) fn new(output_id: OutputId, sequence: u64) -> Self {
        Self {
            output_id,
            sequence,
        }
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
