// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Bounded reuse pool for Vulkan host-visible readback buffers.

use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use super::{VulkanLogicalDevice, VulkanReadbackBuffer, VulkanReadbackError};

/// Current allocation and availability counters for a readback pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VulkanReadbackPoolStats {
    pub allocated_buffers: usize,
    pub allocated_bytes: usize,
    pub idle_buffers: usize,
}

/// Shared manager for a bounded set of staging buffers.
#[derive(Clone)]
pub struct VulkanReadbackBufferPool {
    inner: Arc<VulkanReadbackBufferPoolInner>,
}

struct VulkanReadbackBufferPoolInner {
    device: Arc<VulkanLogicalDevice>,
    max_buffers: usize,
    max_bytes: usize,
    state: Mutex<VulkanReadbackBufferPoolState>,
}

#[derive(Default)]
struct VulkanReadbackBufferPoolState {
    allocated_buffers: usize,
    allocated_bytes: usize,
    idle: Vec<VulkanReadbackBuffer<'static>>,
}

/// A checked-out staging buffer returned to its pool when released.
pub struct VulkanReadbackBufferLease {
    pool: Arc<VulkanReadbackBufferPoolInner>,
    buffer: Option<VulkanReadbackBuffer<'static>>,
}

impl VulkanReadbackBufferPool {
    /// Creates a pool with hard limits on total allocated buffers and bytes.
    pub fn new(
        device: Arc<VulkanLogicalDevice>,
        max_buffers: usize,
        max_bytes: usize,
    ) -> Result<Self, VulkanReadbackError> {
        if max_buffers == 0 || max_bytes == 0 {
            return Err(VulkanReadbackError::InvalidPoolLimits);
        }

        Ok(Self {
            inner: Arc::new(VulkanReadbackBufferPoolInner {
                device,
                max_buffers,
                max_bytes,
                state: Mutex::new(VulkanReadbackBufferPoolState::default()),
            }),
        })
    }

    /// Acquires a matching idle buffer or allocates within the configured limits.
    ///
    /// This method never waits for GPU completion. If all capacity is checked
    /// out, it returns PoolExhausted so the caller can skip capture for this frame.
    pub fn acquire(
        &self,
        width: u32,
        height: u32,
    ) -> Result<VulkanReadbackBufferLease, VulkanReadbackError> {
        let byte_len = readback_byte_len(width, height)?;

        {
            let mut state = self
                .inner
                .state
                .lock()
                .map_err(|_| VulkanReadbackError::PoolLockPoisoned)?;

            if let Some(index) = state
                .idle
                .iter()
                .position(|buffer| buffer.byte_len() == byte_len)
            {
                let buffer = state.idle.swap_remove(index);
                return Ok(VulkanReadbackBufferLease {
                    pool: Arc::clone(&self.inner),
                    buffer: Some(buffer),
                });
            }

            let next_bytes = state
                .allocated_bytes
                .checked_add(byte_len)
                .ok_or(VulkanReadbackError::SizeOverflow)?;
            if state.allocated_buffers >= self.inner.max_buffers
                || next_bytes > self.inner.max_bytes
            {
                return Err(VulkanReadbackError::PoolExhausted);
            }

            // Reserve capacity while the lock is held so concurrent callers
            // cannot exceed the configured memory budget.
            state.allocated_buffers += 1;
            state.allocated_bytes = next_bytes;
        }

        let buffer = match VulkanReadbackBuffer::new_owned(
            Arc::clone(&self.inner.device),
            width,
            height,
        ) {
            Ok(buffer) => buffer,
            Err(error) => {
                if let Ok(mut state) = self.inner.state.lock() {
                    state.allocated_buffers -= 1;
                    state.allocated_bytes -= byte_len;
                }
                return Err(error);
            }
        };

        Ok(VulkanReadbackBufferLease {
            pool: Arc::clone(&self.inner),
            buffer: Some(buffer),
        })
    }

    /// Returns whether this pool belongs to the supplied logical device.
    pub(super) fn owns_device(&self, device: &Arc<VulkanLogicalDevice>) -> bool {
        Arc::ptr_eq(&self.inner.device, device)
    }

    /// Reads pool counters without exposing its internal buffer collection.
    pub fn stats(&self) -> Result<VulkanReadbackPoolStats, VulkanReadbackError> {
        let state = self
            .inner
            .state
            .lock()
            .map_err(|_| VulkanReadbackError::PoolLockPoisoned)?;

        Ok(VulkanReadbackPoolStats {
            allocated_buffers: state.allocated_buffers,
            allocated_bytes: state.allocated_bytes,
            idle_buffers: state.idle.len(),
        })
    }
}

impl Deref for VulkanReadbackBufferLease {
    type Target = VulkanReadbackBuffer<'static>;

    fn deref(&self) -> &Self::Target {
        self.buffer
            .as_ref()
            .expect("readback buffer lease is empty")
    }
}

impl Drop for VulkanReadbackBufferLease {
    fn drop(&mut self) {
        let Some(buffer) = self.buffer.take() else {
            return;
        };

        match self.pool.state.lock() {
            Ok(mut state) => state.idle.push(buffer),
            Err(_) => {
                // A poisoned manager cannot safely accept returned resources.
                // Dropping the buffer still releases its Vulkan allocation.
                drop(buffer);
            }
        }
    }
}

fn readback_byte_len(width: u32, height: u32) -> Result<usize, VulkanReadbackError> {
    if width == 0 || height == 0 {
        return Err(VulkanReadbackError::InvalidDimensions);
    }

    (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(VulkanReadbackError::SizeOverflow)
}

#[cfg(test)]
mod tests {
    use super::readback_byte_len;
    use crate::vulkan::VulkanReadbackError;

    #[test]
    fn readback_size_rejects_zero_dimensions() {
        assert_eq!(
            readback_byte_len(0, 1),
            Err(VulkanReadbackError::InvalidDimensions)
        );
        assert_eq!(
            readback_byte_len(1, 0),
            Err(VulkanReadbackError::InvalidDimensions)
        );
    }

    #[test]
    fn readback_size_uses_four_bytes_per_pixel() {
        assert_eq!(readback_byte_len(16, 8), Ok(512));
    }

    #[test]
    fn readback_size_rejects_overflow() {
        assert_eq!(
            readback_byte_len(u32::MAX, u32::MAX),
            Err(VulkanReadbackError::SizeOverflow)
        );
    }
}
