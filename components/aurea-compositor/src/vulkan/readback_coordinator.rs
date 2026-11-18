// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Bounded coordinator for asynchronous Vulkan readback submissions.

use super::{
    VulkanImageView, VulkanReadbackBufferPool, VulkanReadbackError, VulkanReadbackSubmission,
};

/// Results collected during one non-blocking coordinator poll.
#[derive(Debug, Default)]
pub struct VulkanReadbackPoll {
    /// Frames whose GPU copy and host readback completed successfully.
    pub completed_frames: Vec<Vec<u8>>,
    /// Per-submission errors. Failed submissions remain owned for a later retry
    /// or safe shutdown unless their underlying Vulkan state becomes uncertain.
    pub errors: Vec<VulkanReadbackError>,
}

/// Coordinates a bounded number of in-flight readback submissions.
///
/// All methods that submit, poll or retire work belong on a capture worker.
/// This type never introduces a GPU wait during normal polling. Dropping it
/// with pending work invokes the submission safety fallback and may wait;
/// therefore it must not be owned or dropped by the presentation loop.
pub struct VulkanReadbackCoordinator {
    pool: VulkanReadbackBufferPool,
    max_in_flight: usize,
    submissions: Vec<VulkanReadbackSubmission>,
}

impl VulkanReadbackCoordinator {
    /// Creates a coordinator with a hard in-flight submission limit.
    pub fn new(
        pool: VulkanReadbackBufferPool,
        max_in_flight: usize,
    ) -> Result<Self, VulkanReadbackError> {
        Self::validate_limit(max_in_flight)?;

        Ok(Self {
            pool,
            max_in_flight,
            submissions: Vec::new(),
        })
    }

    fn validate_limit(max_in_flight: usize) -> Result<(), VulkanReadbackError> {
        if max_in_flight == 0 {
            return Err(VulkanReadbackError::InvalidCoordinatorLimit);
        }
        Ok(())
    }

    /// Submits one owned image view if capacity is available.
    ///
    /// A full coordinator returns immediately so the caller can drop this
    /// capture frame and keep the compositor responsive.
    pub fn submit(
        &mut self,
        view: VulkanImageView<'static, 'static>,
    ) -> Result<(), VulkanReadbackError> {
        if self.submissions.len() >= self.max_in_flight {
            return Err(VulkanReadbackError::InFlightLimitReached);
        }

        let submission = VulkanReadbackSubmission::submit(view, &self.pool)?;
        self.submissions.push(submission);
        Ok(())
    }

    /// Retires completed submissions and returns their RGBA8 frames.
    ///
    /// Frames are returned in submission order. Polling stops at the first
    /// incomplete or failed submission so a later frame cannot overtake it.
    /// Fence polling is non-blocking; failed submissions remain owned by this
    /// coordinator and their errors are returned alongside any completed frames.
    pub fn poll_completed(&mut self) -> VulkanReadbackPoll {
        let mut result = VulkanReadbackPoll::default();
        while !self.submissions.is_empty() {
            match self.submissions[0].poll() {
                Ok(false) => break,
                Err(error) => {
                    result.errors.push(error);
                    break;
                }
                Ok(true) => {}
            }

            match self.submissions[0].take_rgba8() {
                Ok(pixels) => {
                    self.submissions.remove(0);
                    result.completed_frames.push(pixels);
                }
                Err(error) => {
                    result.errors.push(error);
                    break;
                }
            }
        }

        result
    }

    /// Number of GPU submissions still owned by this coordinator.
    pub fn in_flight(&self) -> usize {
        self.submissions.len()
    }

    /// Maximum number of simultaneous submissions accepted by this manager.
    pub const fn max_in_flight(&self) -> usize {
        self.max_in_flight
    }

    /// Whether another submission can be accepted without exceeding the cap.
    pub fn has_capacity(&self) -> bool {
        self.submissions.len() < self.max_in_flight
    }
}

#[cfg(test)]
#[path = "readback_coordinator_tests.rs"]
mod tests;
