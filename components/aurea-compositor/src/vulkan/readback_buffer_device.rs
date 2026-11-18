// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Device ownership modes for Vulkan readback allocations.

use std::sync::Arc;

use super::VulkanLogicalDevice;

/// Retains the logical device for a staging allocation.
pub(super) enum VulkanReadbackDevice<'device> {
    Borrowed(&'device VulkanLogicalDevice),
    Shared(Arc<VulkanLogicalDevice>),
}

impl VulkanReadbackDevice<'_> {
    pub(super) fn device(&self) -> &VulkanLogicalDevice {
        match self {
            Self::Borrowed(device) => device,
            Self::Shared(device) => device.as_ref(),
        }
    }
}
