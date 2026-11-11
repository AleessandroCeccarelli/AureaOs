// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan image-view ownership for renderer attachments.

use std::{error::Error, fmt, marker::PhantomData};

use smithay::reexports::ash::vk;

use super::{VulkanImage, VulkanLogicalDevice};
#[cfg(feature = "drm")]
use super::VulkanDrmImage;

/// Errors returned while creating an image view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanImageViewError {
    Creation(vk::Result),
}

impl fmt::Display for VulkanImageViewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Creation(error) => write!(formatter, "Vulkan image-view creation failed: {error:?}"),
        }
    }
}

impl Error for VulkanImageViewError {}

/// Owns a color image view and borrows the image it references.
pub struct VulkanImageView<'image, 'device> {
    device: &'device VulkanLogicalDevice,
    _image: PhantomData<&'image ()>,
    image: vk::Image,
    handle: vk::ImageView,
    format: vk::Format,
    width: u32,
    height: u32,
    usage: vk::ImageUsageFlags,
}

impl<'image, 'device> VulkanImageView<'image, 'device> {
    /// Creates a 2D color view over a Vulkan image.
    pub fn color(
        image: &'image VulkanImage<'device>,
    ) -> Result<Self, VulkanImageViewError> {
        let subresource_range = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);

        let create_info = vk::ImageViewCreateInfo::default()
            .image(image.handle())
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(image.format())
            .subresource_range(subresource_range);

        let device = image.device();
        let handle = unsafe {
            device.handle().create_image_view(&create_info, None)
        }.map_err(VulkanImageViewError::Creation)?;

        Ok(Self {
            device,
            _image: PhantomData,
            image: image.handle(),
            handle,
            format: image.format(),
            width: image.width(),
            height: image.height(),
            usage: image.usage(),
        })
    }

    /// Creates a 2D color view over a modifier-aware DRM export image.
    #[cfg(feature = "drm")]
    pub fn color_drm(
        image: &'image VulkanDrmImage<'device>,
    ) -> Result<Self, VulkanImageViewError> {
        let subresource_range = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);

        let create_info = vk::ImageViewCreateInfo::default()
            .image(image.image())
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(image.format())
            .subresource_range(subresource_range);

        let device = image.device();
        let handle = unsafe {
            device.handle().create_image_view(&create_info, None)
        }.map_err(VulkanImageViewError::Creation)?;

        Ok(Self {
            device,
            _image: PhantomData,
            image: image.image(),
            handle,
            format: image.format(),
            width: image.width(),
            height: image.height(),
            usage: image.usage(),
        })
    }

    pub fn handle(&self) -> vk::ImageView { self.handle }
    pub(super) fn image_handle(&self) -> vk::Image { self.image }
    pub fn format(&self) -> vk::Format { self.format }
    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    pub fn usage(&self) -> vk::ImageUsageFlags { self.usage }
    pub(super) fn device(&self) -> &'device VulkanLogicalDevice { self.device }
}

impl Drop for VulkanImageView<'_, '_> {
    fn drop(&mut self) {
        unsafe {
            self.device.handle().destroy_image_view(self.handle, None);
        }
    }
}
