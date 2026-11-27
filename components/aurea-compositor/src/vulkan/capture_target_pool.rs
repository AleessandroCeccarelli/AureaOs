// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Bounded reuse pool for compositor-owned Vulkan capture targets.

use std::sync::{Arc, Mutex, TryLockError};

use super::{VulkanCaptureTarget, VulkanCaptureTargetError, VulkanLogicalDevice};

/// Snapshot of capture-target pool allocation and availability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VulkanCaptureTargetPoolStats {
    pub allocated_targets: usize,
    pub allocated_bytes: usize,
    pub idle_targets: usize,
}

/// Shared bounded manager for capture-only render targets.
#[derive(Clone)]
pub struct VulkanCaptureTargetPool {
    inner: Arc<VulkanCaptureTargetPoolInner>,
}

struct VulkanCaptureTargetPoolInner {
    device: Arc<VulkanLogicalDevice>,
    max_targets: usize,
    max_bytes: usize,
    state: Mutex<VulkanCaptureTargetPoolState>,
}

#[derive(Default)]
struct VulkanCaptureTargetPoolState {
    allocated_targets: usize,
    allocated_bytes: usize,
    idle: Vec<VulkanCaptureTarget>,
}

impl VulkanCaptureTargetPool {
    /// Creates a pool with hard limits on image count and Vulkan allocation bytes.
    pub fn new(
        device: Arc<VulkanLogicalDevice>,
        max_targets: usize,
        max_bytes: usize,
    ) -> Result<Self, VulkanCaptureTargetError> {
        if max_targets == 0 || max_bytes == 0 {
            return Err(VulkanCaptureTargetError::InvalidPoolLimits);
        }

        Ok(Self {
            inner: Arc::new(VulkanCaptureTargetPoolInner {
                device,
                max_targets,
                max_bytes,
                state: Mutex::new(VulkanCaptureTargetPoolState::default()),
            }),
        })
    }

    /// Allocates one target for the requested size on an initialization/worker thread.
    ///
    /// Idle targets may be evicted to respect the configured limits. Checked-out
    /// targets are never reclaimed while a GPU submission may still reference them.
    pub fn prepare(&self, width: u32, height: u32) -> Result<(), VulkanCaptureTargetError> {
        if self.inner.device.is_unusable() {
            return Err(VulkanCaptureTargetError::DeviceLost);
        }
        if width == 0 || height == 0 {
            return Err(VulkanCaptureTargetError::InvalidDimensions);
        }

        let mut state = self
            .inner
            .state
            .lock()
            .map_err(|_| VulkanCaptureTargetError::PoolLockPoisoned)?;

        // Preparing an already cached size is idempotent. Reusing the idle
        // allocation avoids needless Vulkan allocation/free churn.
        if state
            .idle
            .iter()
            .any(|target| target.width() == width && target.height() == height)
        {
            return Ok(());
        }

        while state.allocated_targets >= self.inner.max_targets {
            let Some(target) = state.idle.pop() else {
                return Err(VulkanCaptureTargetError::PoolExhausted);
            };
            state.allocated_targets -= 1;
            state.allocated_bytes -= target.allocation_size();
            drop(target);
        }

        let target = VulkanCaptureTarget::allocate(Arc::clone(&self.inner.device), width, height)?;
        let allocation_size = target.allocation_size();

        while state
            .allocated_bytes
            .checked_add(allocation_size)
            .is_none_or(|total| total > self.inner.max_bytes)
        {
            if state.idle.is_empty() {
                drop(target);
                return Err(VulkanCaptureTargetError::PoolExhausted);
            }
            let evicted = state.idle.pop().expect("idle target was checked");
            state.allocated_targets -= 1;
            state.allocated_bytes -= evicted.allocation_size();
            drop(evicted);
        }

        state.allocated_targets += 1;
        state.allocated_bytes += allocation_size;
        state.idle.push(target);

        debug_assert!(state.allocated_targets <= self.inner.max_targets);
        debug_assert!(state.allocated_bytes <= self.inner.max_bytes);
        Ok(())
    }

    /// Checks out a matching preallocated target without waiting or allocating.
    ///
    /// A busy or exhausted pool should cause the caller to skip this capture frame.
    pub fn acquire(
        &self,
        width: u32,
        height: u32,
    ) -> Result<VulkanCaptureTargetLease, VulkanCaptureTargetError> {
        if self.inner.device.is_unusable() {
            return Err(VulkanCaptureTargetError::DeviceLost);
        }
        let mut state = match self.inner.state.try_lock() {
            Ok(state) => state,
            Err(TryLockError::Poisoned(_)) => {
                return Err(VulkanCaptureTargetError::PoolLockPoisoned);
            }
            Err(TryLockError::WouldBlock) => {
                return Err(VulkanCaptureTargetError::PoolBusy);
            }
        };

        let Some(index) = state
            .idle
            .iter()
            .position(|target| target.width() == width && target.height() == height)
        else {
            return Err(VulkanCaptureTargetError::PoolExhausted);
        };

        Ok(VulkanCaptureTargetLease {
            pool: Arc::clone(&self.inner),
            target: Some(state.idle.swap_remove(index)),
        })
    }

    /// Reads allocation counters without exposing the idle resource collection.
    pub fn stats(&self) -> Result<VulkanCaptureTargetPoolStats, VulkanCaptureTargetError> {
        let state = self
            .inner
            .state
            .lock()
            .map_err(|_| VulkanCaptureTargetError::PoolLockPoisoned)?;

        Ok(VulkanCaptureTargetPoolStats {
            allocated_targets: state.allocated_targets,
            allocated_bytes: state.allocated_bytes,
            idle_targets: state.idle.len(),
        })
    }
}

/// RAII checkout returned to the pool when its final owner releases it.
pub struct VulkanCaptureTargetLease {
    pool: Arc<VulkanCaptureTargetPoolInner>,
    target: Option<VulkanCaptureTarget>,
}

impl std::ops::Deref for VulkanCaptureTargetLease {
    type Target = VulkanCaptureTarget;

    fn deref(&self) -> &Self::Target {
        self.target.as_ref().expect("capture target lease is empty")
    }
}

impl Drop for VulkanCaptureTargetLease {
    fn drop(&mut self) {
        let Some(target) = self.target.take() else {
            return;
        };

        match self.pool.state.lock() {
            Ok(mut state) => state.idle.push(target),
            Err(_) => {
                // A poisoned pool cannot safely accept returned GPU resources.
                drop(target);
            }
        }
    }
}
