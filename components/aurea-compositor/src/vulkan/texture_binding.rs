// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Owned descriptor pool and sampled-image binding for one compositor texture.

use std::{error::Error, fmt};

use smithay::reexports::ash::vk;

use super::{
    VulkanImageView, VulkanLogicalDevice, VulkanSampler, VulkanTextureDescriptorSetLayout,
};

/// Errors returned while binding a sampled texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanTextureBindingError {
    DeviceMismatch,
    MissingSampledUsage,
    PoolCreation(vk::Result),
    SetAllocation(vk::Result),
}

impl fmt::Display for VulkanTextureBindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeviceMismatch => {
                formatter.write_str("Texture binding resources use different Vulkan devices")
            }
            Self::MissingSampledUsage => formatter.write_str("Texture image lacks sampled usage"),
            Self::PoolCreation(error) => write!(
                formatter,
                "Texture descriptor pool creation failed: {error:?}"
            ),
            Self::SetAllocation(error) => {
                write!(formatter, "Texture descriptor allocation failed: {error:?}")
            }
        }
    }
}

impl Error for VulkanTextureBindingError {}

/// Keeps the image view, sampler and layout alive for the allocated descriptor set.
pub struct VulkanTextureBinding<'resources, 'image, 'device> {
    device: &'device VulkanLogicalDevice,
    _view: &'resources VulkanImageView<'image, 'device>,
    _sampler: &'resources VulkanSampler<'device>,
    _layout: &'resources VulkanTextureDescriptorSetLayout<'device>,
    pool: vk::DescriptorPool,
    set: vk::DescriptorSet,
}

impl<'resources, 'image, 'device> VulkanTextureBinding<'resources, 'image, 'device> {
    pub fn new(
        device: &'device VulkanLogicalDevice,
        layout: &'resources VulkanTextureDescriptorSetLayout<'device>,
        view: &'resources VulkanImageView<'image, 'device>,
        sampler: &'resources VulkanSampler<'device>,
    ) -> Result<Self, VulkanTextureBindingError> {
        if layout.device_handle() != device.handle().handle()
            || view.device().handle().handle() != device.handle().handle()
            || sampler.device_handle() != device.handle().handle()
        {
            return Err(VulkanTextureBindingError::DeviceMismatch);
        }
        if !view.usage().contains(vk::ImageUsageFlags::SAMPLED) {
            return Err(VulkanTextureBindingError::MissingSampledUsage);
        }

        let pool_sizes = [vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(1)
            .pool_sizes(&pool_sizes);
        let pool = unsafe { device.handle().create_descriptor_pool(&pool_info, None) }
            .map_err(VulkanTextureBindingError::PoolCreation)?;

        let layouts = [layout.handle()];
        let allocate_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pool)
            .set_layouts(&layouts);
        let set = match unsafe { device.handle().allocate_descriptor_sets(&allocate_info) } {
            Ok(mut sets) => sets.pop().expect("one descriptor set was requested"),
            Err(error) => {
                unsafe { device.handle().destroy_descriptor_pool(pool, None) };
                return Err(VulkanTextureBindingError::SetAllocation(error));
            }
        };

        let image_info = [vk::DescriptorImageInfo::default()
            .sampler(sampler.handle())
            .image_view(view.handle())
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];
        let writes = [vk::WriteDescriptorSet::default()
            .dst_set(set)
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&image_info)];
        unsafe { device.handle().update_descriptor_sets(&writes, &[]) };

        Ok(Self {
            device,
            _view: view,
            _sampler: sampler,
            _layout: layout,
            pool,
            set,
        })
    }

    pub fn handle(&self) -> vk::DescriptorSet {
        self.set
    }

    /// Returns whether this descriptor set samples the exact supplied image view.
    ///
    /// Matching only the Vulkan device is insufficient: a descriptor set may
    /// legally reference another image on the same device. Callers that record
    /// image barriers for a source view must bind the descriptor for that view.
    pub(super) fn references_view(&self, view: &VulkanImageView<'_, '_>) -> bool {
        self._view.handle() == view.handle()
            && self._view.device().handle().handle() == view.device().handle().handle()
    }

    /// Binds this sampled texture at the requested graphics descriptor-set index.
    pub fn bind(
        &self,
        command_buffer: vk::CommandBuffer,
        pipeline_layout: vk::PipelineLayout,
        set_index: u32,
    ) {
        unsafe {
            self.device.handle().cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline_layout,
                set_index,
                &[self.set],
                &[],
            );
        }
    }
}

impl Drop for VulkanTextureBinding<'_, '_, '_> {
    fn drop(&mut self) {
        unsafe {
            self.device
                .handle()
                .destroy_descriptor_pool(self.pool, None);
        }
    }
}
