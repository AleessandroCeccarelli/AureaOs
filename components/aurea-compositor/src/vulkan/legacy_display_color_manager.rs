// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Per-device cache of shared Vulkan 1.0 display-color passes.

use std::{collections::HashMap, sync::Arc};

use smithay::reexports::ash::vk;

use super::{
    VulkanColorLoad, VulkanDisplayColorPassError, VulkanLegacyRenderPass, VulkanLogicalDevice,
    VulkanSharedLegacyDisplayColorPass,
};

/// Owns one display-color pass per supported output format.
pub(crate) struct VulkanLegacyDisplayColorManager {
    device: Arc<VulkanLogicalDevice>,
    passes: HashMap<vk::Format, Arc<VulkanSharedLegacyDisplayColorPass>>,
}

impl VulkanLegacyDisplayColorManager {
    pub(crate) fn new(device: Arc<VulkanLogicalDevice>) -> Self {
        Self {
            device,
            passes: HashMap::new(),
        }
    }

    pub(crate) fn get(
        &mut self,
        format: vk::Format,
    ) -> Result<Arc<VulkanSharedLegacyDisplayColorPass>, VulkanDisplayColorPassError> {
        if let Some(pass) = self.passes.get(&format) {
            return Ok(Arc::clone(pass));
        }
        let render_pass = Arc::new(
            VulkanLegacyRenderPass::new_shared(
                Arc::clone(&self.device),
                format,
                VulkanColorLoad::DontCare,
            )
            .map_err(|error| VulkanDisplayColorPassError::Pipeline(error.to_string()))?,
        );
        let pass = Arc::new(VulkanSharedLegacyDisplayColorPass::new(
            Arc::clone(&self.device),
            render_pass,
        )?);
        self.passes.insert(format, Arc::clone(&pass));
        Ok(pass)
    }
}
