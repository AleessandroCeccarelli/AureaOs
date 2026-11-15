// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Per-device cache for reusable display-color pipelines.

use std::{collections::HashMap, sync::Arc};

use smithay::reexports::ash::vk;

use super::{VulkanDisplayColorPass, VulkanDisplayColorPassError, VulkanLogicalDevice};

/// Owns one reusable display-color pass per supported scanout format.
pub(crate) struct VulkanDisplayColorManager {
    device: Arc<VulkanLogicalDevice>,
    passes: HashMap<vk::Format, VulkanDisplayColorPass<'static>>,
}

impl VulkanDisplayColorManager {
    pub(crate) fn new(device: Arc<VulkanLogicalDevice>) -> Self {
        Self { device, passes: HashMap::new() }
    }

    /// Returns the cached pass, creating it only on the first request.
    pub(crate) fn pass_for_format(
        &mut self,
        format: vk::Format,
    ) -> Result<&VulkanDisplayColorPass<'static>, VulkanDisplayColorPassError> {
        if !self.passes.contains_key(&format) {
            let pass = VulkanDisplayColorPass::new_shared(Arc::clone(&self.device), format)?;
            self.passes.insert(format, pass);
        }
        self.passes.get(&format).ok_or(VulkanDisplayColorPassError::IncompatibleImages)
    }

    pub(crate) fn cached_format_count(&self) -> usize {
        self.passes.len()
    }
}
