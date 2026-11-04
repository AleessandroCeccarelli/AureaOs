// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan rendering-backend initialization.

use super::{VulkanContext, VulkanInitError, VulkanLogicalDevice};
use crate::renderer::RenderPreference;

/// Owns the Vulkan context and logical device used by the compositor.
pub struct VulkanBackend {
    context: VulkanContext,
    device: VulkanLogicalDevice,
}

impl VulkanBackend {
    /// Initializes Vulkan using the requested GPU preference.
    pub fn new(preference: RenderPreference) -> Result<Self, VulkanInitError> {
        let context = VulkanContext::new()?;
        let device = context.create_logical_device(preference)?;

        Ok(Self { context, device })
    }

    /// Returns the Vulkan context.
    pub fn context(&self) -> &VulkanContext {
        &self.context
    }

    /// Returns the logical device.
    pub fn device(&self) -> &VulkanLogicalDevice {
        &self.device
    }
}
