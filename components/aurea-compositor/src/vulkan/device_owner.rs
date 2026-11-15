// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Shared or borrowed ownership of a Vulkan logical device.

use std::sync::Arc;

use super::VulkanLogicalDevice;

/// Keeps a Vulkan device alive for resources with independent lifetimes.
pub(super) enum VulkanDeviceOwner<'device> {
    Borrowed(&'device VulkanLogicalDevice),
    Shared(Arc<VulkanLogicalDevice>),
}

impl VulkanDeviceOwner<'_> {
    pub(super) fn device(&self) -> &VulkanLogicalDevice {
        match self {
            Self::Borrowed(device) => device,
            Self::Shared(device) => device.as_ref(),
        }
    }
}
