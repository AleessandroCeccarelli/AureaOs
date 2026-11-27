// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan image-view ownership for renderer attachments.

use std::{error::Error, fmt, marker::PhantomData, sync::Arc};

use smithay::reexports::ash::vk;

#[cfg(feature = "drm")]
use super::{VulkanDrmImage, VulkanDmabufImage};
use super::{VulkanImage, VulkanLogicalDevice, VulkanSharedImage};

/// Errors returned while creating an image view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanImageViewError {
    Creation(vk::Result),
}

impl fmt::Display for VulkanImageViewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Creation(error) => {
                write!(formatter, "Vulkan image-view creation failed: {error:?}")
            }
        }
    }
}

impl Error for VulkanImageViewError {}

/// Keeps the logical device alive for the lifetime of an image view.
enum VulkanImageViewDevice<'device> {
    Borrowed(&'device VulkanLogicalDevice),
    Shared(Arc<VulkanSharedImage>),
    #[cfg(feature = "drm")]
    Dmabuf(Arc<VulkanDmabufImage>),
}

impl VulkanImageViewDevice<'_> {
    fn device(&self) -> &VulkanLogicalDevice {
        match self {
            Self::Borrowed(device) => device,
            Self::Shared(image) => image.device(),
            #[cfg(feature = "drm")]
            Self::Dmabuf(image) => image.device(),
        }
    }
}

/// Owns a color image view and retains either its borrowed or shared image owner.
pub struct VulkanImageView<'image, 'device> {
    device: VulkanImageViewDevice<'device>,
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
    pub fn color<'view>(
        image: &'view VulkanImage<'device>,
    ) -> Result<VulkanImageView<'view, 'device>, VulkanImageViewError>
    where
        'device: 'view,
    {
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
        let handle = unsafe { device.handle().create_image_view(&create_info, None) }
            .map_err(VulkanImageViewError::Creation)?;

        Ok(VulkanImageView {
            device: VulkanImageViewDevice::Borrowed(device),
            _image: PhantomData,
            image: image.handle(),
            handle,
            format: image.format(),
            width: image.width(),
            height: image.height(),
            usage: image.usage(),
        })
    }

    /// Creates a 2D color view over a device-owned intermediate image.
    pub fn color_shared(
        image: &'image VulkanSharedImage,
    ) -> Result<VulkanImageView<'image, 'image>, VulkanImageViewError> {
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
        let handle = unsafe { device.handle().create_image_view(&create_info, None) }
            .map_err(VulkanImageViewError::Creation)?;

        Ok(VulkanImageView {
            device: VulkanImageViewDevice::Borrowed(device),
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
        image: &'image VulkanDrmImage,
    ) -> Result<VulkanImageView<'image, 'image>, VulkanImageViewError> {
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
        let handle = unsafe { device.handle().create_image_view(&create_info, None) }
            .map_err(VulkanImageViewError::Creation)?;

        Ok(VulkanImageView {
            device: VulkanImageViewDevice::Borrowed(device),
            _image: PhantomData,
            image: image.image(),
            handle,
            format: image.format(),
            width: image.width(),
            height: image.height(),
            usage: image.usage(),
        })
    }

    /// Creates an owned view over an imported client DMA-BUF image.
    #[cfg(feature = "drm")]
    pub(crate) fn color_dmabuf_owned(
        image: Arc<VulkanDmabufImage>,
    ) -> Result<VulkanImageView<'static, 'static>, VulkanImageViewError> {
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
        let image_handle = image.image();
        let image_format = image.format();
        let image_width = image.width();
        let image_height = image.height();
        let image_usage = image.usage();
        let device = image.device();
        let handle = unsafe { device.handle().create_image_view(&create_info, None) }
            .map_err(VulkanImageViewError::Creation)?;
        Ok(VulkanImageView {
            device: VulkanImageViewDevice::Dmabuf(image),
            _image: PhantomData,
            image: image_handle,
            handle,
            format: image_format,
            width: image_width,
            height: image_height,
            usage: image_usage,
        })
    }

    pub fn handle(&self) -> vk::ImageView {
        self.handle
    }
    pub(super) fn image_handle(&self) -> vk::Image {
        self.image
    }
    pub fn format(&self) -> vk::Format {
        self.format
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn usage(&self) -> vk::ImageUsageFlags {
        self.usage
    }
    pub(super) fn device(&self) -> &VulkanLogicalDevice {
        self.device.device()
    }

    /// Clones the logical-device owner when this view owns a shared image.
    pub(super) fn shared_device_owner(&self) -> Option<Arc<VulkanLogicalDevice>> {
        match &self.device {
            VulkanImageViewDevice::Borrowed(_) => None,
            VulkanImageViewDevice::Shared(image) => Some(image.device_owner()),
            #[cfg(feature = "drm")]
            VulkanImageViewDevice::Dmabuf(_) => None,
        }
    }
}

impl VulkanImageView<'static, 'static> {
    /// Creates an image view that owns a shared reference to its source image.
    ///
    /// This form can be cached independently of the call stack without
    /// allowing the image to be destroyed while the view still exists.
    pub fn color_shared_owned(
        image: Arc<VulkanSharedImage>,
    ) -> Result<VulkanImageView<'static, 'static>, VulkanImageViewError> {
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

        let image_handle = image.handle();
        let image_format = image.format();
        let image_width = image.width();
        let image_height = image.height();
        let image_usage = image.usage();
        let device = image.device();
        let handle = unsafe { device.handle().create_image_view(&create_info, None) }
            .map_err(VulkanImageViewError::Creation)?;

        Ok(VulkanImageView {
            device: VulkanImageViewDevice::Shared(image),
            _image: PhantomData,
            image: image_handle,
            handle,
            format: image_format,
            width: image_width,
            height: image_height,
            usage: image_usage,
        })
    }
}

impl Drop for VulkanImageView<'_, '_> {
    fn drop(&mut self) {
        if self.device.device().is_unusable() { return; }
        unsafe {
            self.device
                .device()
                .handle()
                .destroy_image_view(self.handle, None);
        }
    }
}
