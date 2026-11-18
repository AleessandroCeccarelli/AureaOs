// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Capture Core
// Copyright (C) 2026 Alessandro Ceccarelli

//! Bounded, non-blocking frame queue for capture-to-encoder handoff.

use std::collections::VecDeque;

use crate::{CaptureFrame, CaptureQueueError};

/// A bounded queue that never blocks the compositor or capture producer.
///
/// When full, a newly submitted frame is rejected. The producer can count the
/// drop and continue rendering without waiting for a media encoder.
#[derive(Debug)]
pub struct CaptureFrameQueue {
    capacity: usize,
    queued_bytes: usize,
    byte_limit: usize,
    frames: VecDeque<CaptureFrame>,
    dropped_frames: u64,
}

impl CaptureFrameQueue {
    /// Creates a queue with a fixed non-zero frame capacity.
    pub fn new(capacity: usize, byte_limit: usize) -> Result<Self, CaptureQueueError> {
        if capacity == 0 {
            return Err(CaptureQueueError::ZeroCapacity);
        }

        if byte_limit == 0 {
            return Err(CaptureQueueError::ZeroByteLimit);
        }

        Ok(Self {
            capacity,
            queued_bytes: 0,
            byte_limit,
            frames: VecDeque::with_capacity(capacity),
            dropped_frames: 0,
        })
    }

    /// Enqueues a frame without waiting for the consumer.
    ///
    /// Returns the rejected frame when the queue is full.
    pub fn try_push(&mut self, frame: CaptureFrame) -> Result<(), CaptureFrame> {
        if self.frames.len() >= self.capacity
            || self.queued_bytes.saturating_add(frame.byte_len()) > self.byte_limit
        {
            self.dropped_frames = self.dropped_frames.saturating_add(1);
            return Err(frame);
        }

        self.queued_bytes = self.queued_bytes.saturating_add(frame.byte_len());
        self.frames.push_back(frame);
        Ok(())
    }

    /// Removes the oldest frame for the encoder or image writer.
    pub fn pop(&mut self) -> Option<CaptureFrame> {
        let frame = self.frames.pop_front()?;
        self.queued_bytes = self.queued_bytes.saturating_sub(frame.byte_len());
        Some(frame)
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    pub const fn queued_bytes(&self) -> usize {
        self.queued_bytes
    }

    pub const fn byte_limit(&self) -> usize {
        self.byte_limit
    }

    pub const fn dropped_frames(&self) -> u64 {
        self.dropped_frames
    }
}
