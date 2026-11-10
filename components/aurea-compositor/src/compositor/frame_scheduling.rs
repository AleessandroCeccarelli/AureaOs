
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Frame scheduling operations exposed by the compositor.

use std::{error::Error, fmt};

use crate::{
    frame::{
        FrameLifecycleError,
        FrameRequestResult,
        FrameToken,
    },
    output::OutputId,
};

use super::AureaCompositor;

/// Errors returned when requesting a frame for an output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositorFrameError {
    /// The requested output is not registered.
    UnknownOutput(OutputId),

    /// The requested output is disabled.
    OutputDisabled(OutputId),
}

impl fmt::Display for CompositorFrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownOutput(output_id) => {
                write!(formatter, "Unknown output: {output_id}")
            }
            Self::OutputDisabled(output_id) => {
                write!(formatter, "Output {output_id} is disabled")
            }
        }
    }
}

impl Error for CompositorFrameError {}

impl AureaCompositor {
    /// Requests a frame for a registered and enabled output.
    pub fn request_output_frame(
        &mut self,
        output_id: OutputId,
    ) -> Result<FrameRequestResult, CompositorFrameError> {
        let output = self
            .output_manager
            .registry()
            .get(output_id)
            .ok_or(CompositorFrameError::UnknownOutput(output_id))?;

        if !output.is_enabled() {
            return Err(CompositorFrameError::OutputDisabled(output_id));
        }

        Ok(self.frame_scheduler.request_frame(output_id))
    }

    /// Begins a pending frame for an enabled output.
    ///
    /// Returns `None` when no request is waiting.
    pub fn begin_output_frame(
        &mut self,
        output_id: OutputId,
    ) -> Option<FrameToken> {
        let Some(output) = self.output_manager.registry().get(output_id) else {
            self.frame_scheduler.remove_output(output_id);
            return None;
        };

        if !output.is_enabled() {
            self.frame_scheduler.cancel_request(output_id);
            return None;
        }

        self.frame_scheduler.begin_frame(output_id)
    }

    /// Completes an active frame.
    pub fn complete_output_frame(
        &mut self,
        token: FrameToken,
    ) -> Result<(), FrameLifecycleError> {
        self.frame_scheduler.complete_frame(token)
    }

    /// Cancels a pending frame request.
    ///
    /// An already active frame cannot be cancelled.
    pub fn cancel_output_frame(&mut self, output_id: OutputId) -> bool {
        self.frame_scheduler.cancel_request(output_id)
    }

    /// Returns whether an output has a pending frame.
    pub fn has_pending_output_frame(&self, output_id: OutputId) -> bool {
        self.frame_scheduler.has_pending_frame(output_id)
    }

    /// Returns whether an output has an active frame.
    pub fn has_active_output_frame(&self, output_id: OutputId) -> bool {
        self.frame_scheduler.has_frame_in_flight(output_id)
    }
}
