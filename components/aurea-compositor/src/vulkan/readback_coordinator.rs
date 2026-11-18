// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Bounded coordinator for asynchronous Vulkan readback submissions.

use super::{
    VulkanImageView, VulkanReadbackBufferPool, VulkanReadbackError,
    VulkanReadbackSubmission,
};

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
        if max_in_flight == 0 {
            return Err(VulkanReadbackError::InvalidCoordinatorLimit);
        }

        Ok(Self {
            pool,
            max_in_flight,
            submissions: Vec::with_capacity(max_in_flight),
        })
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
    /// Incomplete submissions remain owned by this coordinator. Fence polling
    /// is non-blocking; a fence or mapping error is reported without silently
    /// discarding the affected submission.
    pub fn poll_completed(&mut self) -> Result<Vec<Vec<u8>>, VulkanReadbackError> {
        let mut completed_frames = Vec::new();
        let mut index = 0;

        while index < self.submissions.len() {
            if !self.submissions[index].poll()? {
                index += 1;
                continue;
            }

            let mut submission = self.submissions.swap_remove(index);
            let pixels = submission.take_rgba8()?;
            completed_frames.push(pixels);
        }

        Ok(completed_frames)
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
