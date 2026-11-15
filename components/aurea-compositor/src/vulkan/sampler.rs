// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan sampler ownership for compositor textures.

use std::{error::Error, fmt, sync::Arc};

use smithay::reexports::ash::vk;

use super::{VulkanDeviceOwner, VulkanLogicalDevice};

/// Errors returned while creating a Vulkan sampler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanSamplerError {
    Creation(vk::Result),
}

impl fmt::Display for VulkanSamplerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Creation(error) => write!(formatter, "Vulkan sampler creation failed: {error:?}"),
        }
    }
}

impl Error for VulkanSamplerError {}

/// Owns a linear, clamp-to-edge sampler and its device lifetime.
pub struct VulkanSampler<'device> {
    device: VulkanDeviceOwner<'device>,
    handle: vk::Sampler,
}

impl<'device> VulkanSampler<'device> {
    pub fn linear_clamp(device: &'device VulkanLogicalDevice) -> Result<Self, VulkanSamplerError> {
        Self::create(VulkanDeviceOwner::Borrowed(device))
    }

    /// Creates a sampler that independently retains its logical device.
    pub fn linear_clamp_shared(
        device: Arc<VulkanLogicalDevice>,
    ) -> Result<VulkanSampler<'static>, VulkanSamplerError> {
        Self::create(VulkanDeviceOwner::Shared(device))
    }

    fn create<'owner>(
        device: VulkanDeviceOwner<'owner>,
    ) -> Result<VulkanSampler<'owner>, VulkanSamplerError> {
        let info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .mip_lod_bias(0.0)
            .anisotropy_enable(false)
            .compare_enable(false)
            .min_lod(0.0)
            .max_lod(0.0)
            .border_color(vk::BorderColor::FLOAT_TRANSPARENT_BLACK)
            .unnormalized_coordinates(false);

        let handle = unsafe { device.device().handle().create_sampler(&info, None) }
            .map_err(VulkanSamplerError::Creation)?;

        Ok(VulkanSampler { device, handle })
    }

    pub fn handle(&self) -> vk::Sampler {
        self.handle
    }

    pub(super) fn device_handle(&self) -> vk::Device {
        self.device.device().handle().handle()
    }
}

impl Drop for VulkanSampler<'_> {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device()
                .handle()
                .destroy_sampler(self.handle, None);
        }
    }
}
