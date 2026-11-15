// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Descriptor-set layout for sampled compositor textures.

use std::{error::Error, fmt, sync::Arc};

use smithay::reexports::ash::vk;

use super::{VulkanDeviceOwner, VulkanLogicalDevice};

/// Errors returned while creating a texture descriptor-set layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanTextureLayoutError {
    Creation(vk::Result),
}

impl fmt::Display for VulkanTextureLayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Creation(error) => write!(
                formatter,
                "Texture descriptor layout creation failed: {error:?}"
            ),
        }
    }
}

impl Error for VulkanTextureLayoutError {}

/// Binding zero contains one combined image sampler visible to fragment shaders.
pub struct VulkanTextureDescriptorSetLayout<'device> {
    device: VulkanDeviceOwner<'device>,
    handle: vk::DescriptorSetLayout,
}

impl<'device> VulkanTextureDescriptorSetLayout<'device> {
    pub fn new(device: &'device VulkanLogicalDevice) -> Result<Self, VulkanTextureLayoutError> {
        Self::create(VulkanDeviceOwner::Borrowed(device))
    }

    /// Creates a descriptor layout that independently retains its logical device.
    pub fn new_shared(
        device: Arc<VulkanLogicalDevice>,
    ) -> Result<VulkanTextureDescriptorSetLayout<'static>, VulkanTextureLayoutError> {
        Self::create(VulkanDeviceOwner::Shared(device))
    }

    fn create<'owner>(
        device: VulkanDeviceOwner<'owner>,
    ) -> Result<VulkanTextureDescriptorSetLayout<'owner>, VulkanTextureLayoutError> {
        let bindings = [vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)];
        let info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        let handle = unsafe {
            device
                .device()
                .handle()
                .create_descriptor_set_layout(&info, None)
        }
        .map_err(VulkanTextureLayoutError::Creation)?;

        Ok(Self { device, handle })
    }

    pub fn handle(&self) -> vk::DescriptorSetLayout {
        self.handle
    }

    pub(super) fn device_handle(&self) -> vk::Device {
        self.device.device().handle().handle()
    }
}

impl Drop for VulkanTextureDescriptorSetLayout<'_> {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device()
                .handle()
                .destroy_descriptor_set_layout(self.handle, None);
        }
    }
}
