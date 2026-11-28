// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Per-device cache of reusable Vulkan 1.0 render passes.

use std::{collections::HashMap, sync::Arc};

use smithay::reexports::ash::vk;

use super::{
    VulkanColorLoad, VulkanLegacyRenderPass, VulkanLegacyRenderPassError, VulkanLogicalDevice,
};

/// Owns persistent legacy render passes keyed by attachment format and load operation.
///
/// Framebuffer targets are intentionally not cached here: they borrow a render
/// pass and an image view and must remain scoped to the lifetime of that image.
pub(crate) struct VulkanLegacyRenderPassManager {
    device: Arc<VulkanLogicalDevice>,
    clear_passes: HashMap<vk::Format, Arc<VulkanLegacyRenderPass<'static>>>,
    load_passes: HashMap<vk::Format, Arc<VulkanLegacyRenderPass<'static>>>,
    discard_passes: HashMap<vk::Format, Arc<VulkanLegacyRenderPass<'static>>>,
}

impl VulkanLegacyRenderPassManager {
    pub(crate) fn new(device: Arc<VulkanLogicalDevice>) -> Self {
        Self {
            device,
            clear_passes: HashMap::new(),
            load_passes: HashMap::new(),
            discard_passes: HashMap::new(),
        }
    }

    pub(crate) fn clear(
        &mut self,
        format: vk::Format,
    ) -> Result<Arc<VulkanLegacyRenderPass<'static>>, VulkanLegacyRenderPassError> {
        Self::get_or_create(
            &self.device,
            &mut self.clear_passes,
            format,
            VulkanColorLoad::Clear([0.0; 4]),
        )
    }

    pub(crate) fn load(
        &mut self,
        format: vk::Format,
    ) -> Result<Arc<VulkanLegacyRenderPass<'static>>, VulkanLegacyRenderPassError> {
        Self::get_or_create(
            &self.device,
            &mut self.load_passes,
            format,
            VulkanColorLoad::Load,
        )
    }

    pub(crate) fn discard(
        &mut self,
        format: vk::Format,
    ) -> Result<Arc<VulkanLegacyRenderPass<'static>>, VulkanLegacyRenderPassError> {
        Self::get_or_create(
            &self.device,
            &mut self.discard_passes,
            format,
            VulkanColorLoad::DontCare,
        )
    }

    fn get_or_create(
        device: &Arc<VulkanLogicalDevice>,
        cache: &mut HashMap<vk::Format, Arc<VulkanLegacyRenderPass<'static>>>,
        format: vk::Format,
        load: VulkanColorLoad,
    ) -> Result<Arc<VulkanLegacyRenderPass<'static>>, VulkanLegacyRenderPassError> {
        if let Some(pass) = cache.get(&format) {
            return Ok(Arc::clone(pass));
        }
        let pass = Arc::new(VulkanLegacyRenderPass::new_shared(
            Arc::clone(device),
            format,
            load,
        )?);
        cache.insert(format, Arc::clone(&pass));
        Ok(pass)
    }
}
