// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan descriptor-pool arena and sampled-image bindings.

use std::{error::Error, fmt, sync::Arc};

use smithay::reexports::ash::vk;

use super::{
    VulkanImageView, VulkanLogicalDevice, VulkanSampler, VulkanTextureDescriptorSetLayout,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanTextureBindingError {
    DeviceMismatch,
    MissingSampledUsage,
    InvalidPoolCapacity,
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
            Self::InvalidPoolCapacity => {
                formatter.write_str("Texture descriptor pool capacity must be greater than zero")
            }
            Self::PoolCreation(error) => {
                write!(
                    formatter,
                    "Texture descriptor pool creation failed: {error:?}"
                )
            }
            Self::SetAllocation(error) => {
                write!(formatter, "Texture descriptor allocation failed: {error:?}")
            }
        }
    }
}

impl Error for VulkanTextureBindingError {}

/// One descriptor pool shared by all sampled textures in a frame.
pub struct VulkanTextureBindingPool {
    device: Arc<VulkanLogicalDevice>,
    pool: vk::DescriptorPool,
}

impl VulkanTextureBindingPool {
    pub fn new(
        device: Arc<VulkanLogicalDevice>,
        max_sets: u32,
    ) -> Result<Self, VulkanTextureBindingError> {
        if max_sets == 0 {
            return Err(VulkanTextureBindingError::InvalidPoolCapacity);
        }

        let pool_sizes = [vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(max_sets)];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(max_sets)
            .pool_sizes(&pool_sizes);
        let pool = unsafe { device.handle().create_descriptor_pool(&pool_info, None) }
            .map_err(VulkanTextureBindingError::PoolCreation)?;

        Ok(Self { device, pool })
    }

    fn allocate_set(
        &self,
        layout: &VulkanTextureDescriptorSetLayout<'_>,
    ) -> Result<vk::DescriptorSet, VulkanTextureBindingError> {
        if layout.device_handle() != self.device.handle().handle() {
            return Err(VulkanTextureBindingError::DeviceMismatch);
        }

        let layouts = [layout.handle()];
        let info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.pool)
            .set_layouts(&layouts);
        let mut sets = unsafe { self.device.handle().allocate_descriptor_sets(&info) }
            .map_err(VulkanTextureBindingError::SetAllocation)?;
        Ok(sets.pop().expect("one descriptor set was requested"))
    }

    fn device_handle(&self) -> vk::Device {
        self.device.handle().handle()
    }
}

impl Drop for VulkanTextureBindingPool {
    fn drop(&mut self) {
        if self.device.is_unusable() {
            return;
        }
        unsafe {
            self.device
                .handle()
                .destroy_descriptor_pool(self.pool, None)
        };
    }
}

/// Keeps all resources referenced by one sampled-image descriptor set alive.
pub struct VulkanTextureBinding<'resources, 'image, 'image_device, 'resource_device> {
    device: &'image_device VulkanLogicalDevice,
    _view: &'resources VulkanImageView<'image, 'image_device>,
    _sampler: &'resources VulkanSampler<'resource_device>,
    _layout: &'resources VulkanTextureDescriptorSetLayout<'resource_device>,
    _shared_pool: Option<&'resources VulkanTextureBindingPool>,
    pool: vk::DescriptorPool,
    owns_pool: bool,
    set: vk::DescriptorSet,
}

impl<'resources, 'image, 'image_device, 'resource_device>
    VulkanTextureBinding<'resources, 'image, 'image_device, 'resource_device>
{
    /// Creates a standalone binding with its own one-set descriptor pool.
    pub fn new(
        device: &'image_device VulkanLogicalDevice,
        layout: &'resources VulkanTextureDescriptorSetLayout<'resource_device>,
        view: &'resources VulkanImageView<'image, 'image_device>,
        sampler: &'resources VulkanSampler<'resource_device>,
    ) -> Result<Self, VulkanTextureBindingError> {
        Self::validate_resources(device, layout, view, sampler)?;

        let pool_sizes = [vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(1)
            .pool_sizes(&pool_sizes);
        let pool = unsafe { device.handle().create_descriptor_pool(&pool_info, None) }
            .map_err(VulkanTextureBindingError::PoolCreation)?;

        let layouts = [layout.handle()];
        let info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pool)
            .set_layouts(&layouts);
        let set = match unsafe { device.handle().allocate_descriptor_sets(&info) } {
            Ok(mut sets) => sets.pop().expect("one descriptor set was requested"),
            Err(error) => {
                unsafe { device.handle().destroy_descriptor_pool(pool, None) };
                return Err(VulkanTextureBindingError::SetAllocation(error));
            }
        };

        Self::write_descriptor(device, set, view, sampler);
        Ok(Self {
            device,
            _view: view,
            _sampler: sampler,
            _layout: layout,
            _shared_pool: None,
            pool,
            owns_pool: true,
            set,
        })
    }

    /// Allocates a descriptor set from a shared frame-scoped pool.
    pub fn new_in_pool(
        device: &'image_device VulkanLogicalDevice,
        pool: &'resources VulkanTextureBindingPool,
        layout: &'resources VulkanTextureDescriptorSetLayout<'resource_device>,
        view: &'resources VulkanImageView<'image, 'image_device>,
        sampler: &'resources VulkanSampler<'resource_device>,
    ) -> Result<Self, VulkanTextureBindingError> {
        Self::validate_resources(device, layout, view, sampler)?;
        if pool.device_handle() != device.handle().handle() {
            return Err(VulkanTextureBindingError::DeviceMismatch);
        }

        let set = pool.allocate_set(layout)?;
        Self::write_descriptor(device, set, view, sampler);
        Ok(Self {
            device,
            _view: view,
            _sampler: sampler,
            _layout: layout,
            _shared_pool: Some(pool),
            pool: pool.pool,
            owns_pool: false,
            set,
        })
    }

    fn validate_resources(
        device: &VulkanLogicalDevice,
        layout: &VulkanTextureDescriptorSetLayout<'_>,
        view: &VulkanImageView<'_, '_>,
        sampler: &VulkanSampler<'_>,
    ) -> Result<(), VulkanTextureBindingError> {
        if layout.device_handle() != device.handle().handle()
            || view.device().handle().handle() != device.handle().handle()
            || sampler.device_handle() != device.handle().handle()
        {
            return Err(VulkanTextureBindingError::DeviceMismatch);
        }
        if !view.usage().contains(vk::ImageUsageFlags::SAMPLED) {
            return Err(VulkanTextureBindingError::MissingSampledUsage);
        }
        Ok(())
    }

    fn write_descriptor(
        device: &VulkanLogicalDevice,
        set: vk::DescriptorSet,
        view: &VulkanImageView<'_, '_>,
        sampler: &VulkanSampler<'_>,
    ) {
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
    }

    pub fn handle(&self) -> vk::DescriptorSet {
        self.set
    }

    /// Returns whether this descriptor set samples the exact supplied image view.
    ///
    /// Matching only the Vulkan device is insufficient: a descriptor set may
    /// legally reference another image on the same device.
    pub(super) fn references_view(&self, view: &VulkanImageView<'_, '_>) -> bool {
        self._view.handle() == view.handle()
            && self._view.device().handle().handle() == view.device().handle().handle()
    }

    /// Returns whether this descriptor set was allocated with the exact layout.
    pub(super) fn uses_layout(&self, layout: &VulkanTextureDescriptorSetLayout<'_>) -> bool {
        self._layout.handle() == layout.handle()
            && self._layout.device_handle() == layout.device_handle()
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

impl Drop for VulkanTextureBinding<'_, '_, '_, '_> {
    fn drop(&mut self) {
        if self.device.is_unusable() {
            return;
        }
        if self.owns_pool {
            unsafe {
                self.device
                    .handle()
                    .destroy_descriptor_pool(self.pool, None);
            }
        }
    }
}
