// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Bounded reuse pool for Vulkan host-visible readback buffers.

use std::sync::{Arc, Mutex, TryLockError};

use super::{
    VulkanLogicalDevice, VulkanReadbackBuffer, VulkanReadbackBufferLease, VulkanReadbackError,
};

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

pub(super) struct VulkanReadbackBufferPoolInner {
    device: Arc<VulkanLogicalDevice>,
    max_buffers: usize,
    max_bytes: usize,
    pub(super) state: Mutex<VulkanReadbackBufferPoolState>,
}

#[derive(Default)]
pub(super) struct VulkanReadbackBufferPoolState {
    allocated_buffers: usize,
    allocated_bytes: usize,
    pub(super) idle: Vec<VulkanReadbackBuffer<'static>>,
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

    /// Allocates one staging buffer on a worker and makes it available for capture.
    ///
    /// Call during initialization or from a background capture worker, never
    /// from the compositor presentation path. Repeated calls can prewarm a pool.
    pub fn prepare(&self, width: u32, height: u32) -> Result<(), VulkanReadbackError> {
        if self.inner.device.is_unusable() {
            return Err(VulkanReadbackError::Submission(
                smithay::reexports::ash::vk::Result::ERROR_DEVICE_LOST,
            ));
        }
        readback_byte_len(width, height)?;
        let mut state = self
            .inner
            .state
            .lock()
            .map_err(|_| VulkanReadbackError::PoolLockPoisoned)?;

        while state.allocated_buffers >= self.inner.max_buffers {
            let Some(buffer) = state.idle.pop() else {
                return Err(VulkanReadbackError::PoolExhausted);
            };
            state.allocated_buffers -= 1;
            state.allocated_bytes -= buffer.allocation_size();
            drop(buffer);
        }

        let available_bytes = self
            .inner
            .max_bytes
            .checked_sub(state.allocated_bytes)
            .ok_or(VulkanReadbackError::PoolExhausted)?;

        let buffer = match VulkanReadbackBuffer::new_owned(
            Arc::clone(&self.inner.device),
            width,
            height,
            available_bytes,
        ) {
            Ok(buffer) => buffer,
            Err(VulkanReadbackError::PoolExhausted) => {
                // Evict idle allocations only. Checked-out leases remain owned
                // by their submissions and are never reclaimed prematurely.
                while let Some(buffer) = state.idle.pop() {
                    state.allocated_buffers -= 1;
                    state.allocated_bytes -= buffer.allocation_size();
                    drop(buffer);
                }

                let available_bytes = self
                    .inner
                    .max_bytes
                    .checked_sub(state.allocated_bytes)
                    .ok_or(VulkanReadbackError::PoolExhausted)?;
                VulkanReadbackBuffer::new_owned(
                    Arc::clone(&self.inner.device),
                    width,
                    height,
                    available_bytes,
                )?
            }
            Err(error) => return Err(error),
        };

        let allocation_size = buffer.allocation_size();
        state.allocated_buffers += 1;
        state.allocated_bytes += allocation_size;
        state.idle.push(buffer);

        debug_assert!(state.allocated_buffers <= self.inner.max_buffers);
        debug_assert!(state.allocated_bytes <= self.inner.max_bytes);
        Ok(())
    }

    /// Checks out a matching preallocated buffer without waiting or allocating.
    ///
    /// This method is suitable for a capture worker's frame path. When no
    /// matching buffer is idle, callers should skip this capture frame.
    pub fn acquire(
        &self,
        width: u32,
        height: u32,
    ) -> Result<VulkanReadbackBufferLease, VulkanReadbackError> {
        if self.inner.device.is_unusable() {
            return Err(VulkanReadbackError::Submission(
                smithay::reexports::ash::vk::Result::ERROR_DEVICE_LOST,
            ));
        }
        let byte_len = readback_byte_len(width, height)?;
        let mut state = match self.inner.state.try_lock() {
            Ok(state) => state,
            Err(TryLockError::Poisoned(_)) => {
                return Err(VulkanReadbackError::PoolLockPoisoned);
            }
            Err(TryLockError::WouldBlock) => {
                return Err(VulkanReadbackError::PoolBusy);
            }
        };

        let Some(index) = state
            .idle
            .iter()
            .position(|buffer| buffer.byte_len() == byte_len)
        else {
            return Err(VulkanReadbackError::PoolExhausted);
        };

        let buffer = state.idle.swap_remove(index);
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
#[path = "readback_buffer_pool_tests.rs"]
mod tests;
