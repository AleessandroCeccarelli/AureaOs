// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! RAII lease for a checked-out Vulkan readback staging buffer.

use std::{ops::Deref, sync::Arc};

use super::{VulkanReadbackBuffer, readback_buffer_pool::VulkanReadbackBufferPoolInner};

/// A checked-out staging buffer returned to its pool when released.
pub struct VulkanReadbackBufferLease {
    pub(super) pool: Arc<VulkanReadbackBufferPoolInner>,
    pub(super) buffer: Option<VulkanReadbackBuffer<'static>>,
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
