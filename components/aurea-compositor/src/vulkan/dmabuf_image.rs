// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan import of single-plane client DMA-BUF images.

use std::{
    error::Error,
    fmt,
    os::fd::{AsRawFd, IntoRawFd},
    sync::Arc,
};

use smithay::reexports::ash::{khr, vk};

use super::{
    DmabufImportLayout, VulkanDmabufImportSupport, VulkanDmabufImportSupportError,
    VulkanLogicalDevice, memory_type::select_memory_type_index,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VulkanDmabufImageError {
    Unsupported(VulkanDmabufImportSupportError),
    InvalidPlaneCount,
    FileDescriptorDuplication,
    FileDescriptorProperties(vk::Result),
    MemoryTypeUnavailable,
    ImageCreation(vk::Result),
    MemoryAllocation(vk::Result),
    MemoryBinding(vk::Result),
}

impl fmt::Display for VulkanDmabufImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DMA-BUF Vulkan import failed: {self:?}")
    }
}
impl Error for VulkanDmabufImageError {}

/// Owns an imported image and retains the source DMA-BUF until GPU use ends.
///
/// This first importer intentionally accepts one memory plane only. It does
/// not claim support for multi-planar formats or implicit producer sync.
pub(crate) struct VulkanDmabufImage {
    device: Arc<VulkanLogicalDevice>,
    image: vk::Image,
    memory: vk::DeviceMemory,
    _layout: DmabufImportLayout,
    format: vk::Format,
    width: u32,
    height: u32,
}

impl VulkanDmabufImage {
    pub(crate) fn import(
        device: Arc<VulkanLogicalDevice>,
        layout: DmabufImportLayout,
    ) -> Result<Self, VulkanDmabufImageError> {
        let support = VulkanDmabufImportSupport::query(&device, &layout)
            .map_err(VulkanDmabufImageError::Unsupported)?;
        let plane = layout
            .planes()
            .first()
            .ok_or(VulkanDmabufImageError::InvalidPlaneCount)?;
        let borrowed_fd = layout
            .plane_handles()
            .next()
            .ok_or(VulkanDmabufImageError::InvalidPlaneCount)?;
        let imported_fd = borrowed_fd
            .try_clone_to_owned()
            .map_err(|_| VulkanDmabufImageError::FileDescriptorDuplication)?;

        let plane_layouts = [vk::SubresourceLayout::default()
            .offset(u64::from(plane.offset))
            .row_pitch(u64::from(plane.stride))];
        let mut explicit_modifier = vk::ImageDrmFormatModifierExplicitCreateInfoEXT::default()
            .drm_format_modifier(support.modifier)
            .plane_layouts(&plane_layouts);
        let mut external_image = vk::ExternalMemoryImageCreateInfo::default()
            .handle_types(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(support.format)
            .extent(vk::Extent3D {
                width: support.width,
                height: support.height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT)
            .usage(vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .push_next(&mut explicit_modifier)
            .push_next(&mut external_image);

        let raw = device.handle();
        let image = unsafe { raw.create_image(&image_info, None) }.map_err(|error| {
            if error == vk::Result::ERROR_DEVICE_LOST {
                device.mark_unusable();
            }
            VulkanDmabufImageError::ImageCreation(error)
        })?;
        let requirements = unsafe { raw.get_image_memory_requirements(image) };

        let fd_loader = khr::external_memory_fd::Device::new(device.instance(), raw);
        let mut fd_properties = vk::MemoryFdPropertiesKHR::default();
        unsafe {
            fd_loader.get_memory_fd_properties(
                vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT,
                imported_fd.as_raw_fd(),
                &mut fd_properties,
            )
        }
        .map_err(|error| {
            if error == vk::Result::ERROR_DEVICE_LOST {
                device.mark_unusable();
            }
            unsafe {
                raw.destroy_image(image, None);
            }
            VulkanDmabufImageError::FileDescriptorProperties(error)
        })?;

        let compatible_types = requirements.memory_type_bits & fd_properties.memory_type_bits;
        let Some(memory_type_index) = select_memory_type_index(
            device.memory_properties(),
            compatible_types,
            vk::MemoryPropertyFlags::empty(),
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        ) else {
            unsafe {
                raw.destroy_image(image, None);
            }
            return Err(VulkanDmabufImageError::MemoryTypeUnavailable);
        };

        let mut import_info = vk::ImportMemoryFdInfoKHR::default()
            .handle_type(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT)
            .fd(imported_fd.as_raw_fd());
        let mut dedicated_info = vk::MemoryDedicatedAllocateInfo::default().image(image);
        let allocation_info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type_index)
            .push_next(&mut import_info)
            .push_next(&mut dedicated_info);

        // Vulkan takes ownership of the duplicated FD only when allocation
        // succeeds. On failure, imported_fd remains owned here and is closed.
        let memory = match unsafe { raw.allocate_memory(&allocation_info, None) } {
            Ok(memory) => {
                let _raw_fd_owned_by_vulkan = imported_fd.into_raw_fd();
                memory
            }
            Err(error) => {
                if error == vk::Result::ERROR_DEVICE_LOST {
                    device.mark_unusable();
                } else {
                    unsafe {
                        raw.destroy_image(image, None);
                    }
                }
                return Err(VulkanDmabufImageError::MemoryAllocation(error));
            }
        };

        if let Err(error) = unsafe { raw.bind_image_memory(image, memory, 0) } {
            if error == vk::Result::ERROR_DEVICE_LOST {
                device.mark_unusable();
            } else {
                unsafe {
                    raw.free_memory(memory, None);
                    raw.destroy_image(image, None);
                }
            }
            return Err(VulkanDmabufImageError::MemoryBinding(error));
        }

        Ok(Self {
            device,
            image,
            memory,
            _layout: layout,
            format: support.format,
            width: support.width,
            height: support.height,
        })
    }

    pub(crate) fn image(&self) -> vk::Image {
        self.image
    }
    pub(crate) fn format(&self) -> vk::Format {
        self.format
    }
    pub(crate) fn width(&self) -> u32 {
        self.width
    }
    pub(crate) fn height(&self) -> u32 {
        self.height
    }
    pub(crate) fn usage(&self) -> vk::ImageUsageFlags {
        vk::ImageUsageFlags::SAMPLED
    }
    pub(crate) fn device(&self) -> &VulkanLogicalDevice {
        &self.device
    }
}

impl Drop for VulkanDmabufImage {
    fn drop(&mut self) {
        if self.device.is_unusable() {
            return;
        }
        unsafe {
            self.device.handle().destroy_image(self.image, None);
            self.device.handle().free_memory(self.memory, None);
        }
    }
}
