// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan integration for the AureaOS compositor.

mod backend;
mod command_buffer;
mod context;
mod external_memory;
mod logical_device;
mod physical_device;
mod queue;
mod queue_submitter;

use std::{error::Error, fmt};

/// Error returned while initializing Vulkan.
#[derive(Debug)]
pub struct VulkanInitError {
    message: String,
}

impl VulkanInitError {
    pub(super) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for VulkanInitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for VulkanInitError {}

pub use backend::VulkanBackend;
pub use command_buffer::VulkanCommandBuffer;
pub use context::VulkanContext;
pub use external_memory::VulkanExternalMemoryCapabilities;
pub use logical_device::VulkanLogicalDevice;
pub use physical_device::{VulkanDeviceInfo, VulkanDeviceType};
pub use queue::VulkanQueueFamilyInfo;

pub(super) use queue_submitter::VulkanQueueSubmitter;
