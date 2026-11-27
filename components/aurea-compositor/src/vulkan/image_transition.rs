// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan 1.0-compatible explicit image transitions for color images.

use std::{error::Error, fmt};

use smithay::reexports::ash::vk;

use super::VulkanLogicalDevice;

/// Image states used by the initial color-compositing path.
///
/// These states describe access intent, not automatic resource tracking.
/// The caller must provide the actual previous layout for the submitted work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanImageState {
    Undefined,
    ColorAttachment,
    ShaderRead,
    TransferSource,
    TransferDestination,
    General,
}

impl VulkanImageState {
    fn layout(self) -> vk::ImageLayout {
        match self {
            Self::Undefined => vk::ImageLayout::UNDEFINED,
            Self::ColorAttachment => vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            Self::ShaderRead => vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            Self::TransferSource => vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            Self::TransferDestination => vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            Self::General => vk::ImageLayout::GENERAL,
        }
    }

    fn stage(self) -> vk::PipelineStageFlags {
        match self {
            Self::Undefined => vk::PipelineStageFlags::TOP_OF_PIPE,
            Self::ColorAttachment => vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            Self::ShaderRead => vk::PipelineStageFlags::FRAGMENT_SHADER,
            Self::TransferSource | Self::TransferDestination => vk::PipelineStageFlags::TRANSFER,
            Self::General => vk::PipelineStageFlags::ALL_COMMANDS,
        }
    }

    fn access(self) -> vk::AccessFlags {
        match self {
            Self::Undefined => vk::AccessFlags::empty(),
            Self::ColorAttachment => {
                vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE
            }
            Self::ShaderRead => vk::AccessFlags::SHADER_READ,
            Self::TransferSource => vk::AccessFlags::TRANSFER_READ,
            Self::TransferDestination => vk::AccessFlags::TRANSFER_WRITE,
            Self::General => vk::AccessFlags::MEMORY_READ | vk::AccessFlags::MEMORY_WRITE,
        }
    }
}

/// Errors returned while recording an external ownership transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanImageTransitionError {
    ForeignQueueFamilyUnsupported,
}

impl fmt::Display for VulkanImageTransitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ForeignQueueFamilyUnsupported => formatter
                .write_str("VK_EXT_queue_family_foreign is not enabled on this Vulkan device"),
        }
    }
}

impl Error for VulkanImageTransitionError {}

/// Records image barriers using the Vulkan 1.0 synchronization API.
pub struct VulkanImageTransition;

impl VulkanImageTransition {
    /// Releases a DMA-BUF-backed image from the graphics queue to KMS/foreign ownership.
    ///
    /// The image is transitioned to GENERAL before the external consumer accesses
    /// the DMA-BUF. Submit this barrier on the graphics queue and signal the
    /// sync-file fence that is attached to the KMS plane as IN_FENCE_FD.
    ///
    /// The caller must not submit this operation unless the image was created
    /// with exportable DMA-BUF memory and the device enabled
    /// VK_EXT_queue_family_foreign.
    pub fn record_release_to_foreign(
        device: &VulkanLogicalDevice,
        command_buffer: vk::CommandBuffer,
        image: vk::Image,
        old_state: VulkanImageState,
    ) -> Result<(), VulkanImageTransitionError> {
        if !device.supports_foreign_queue_family() {
            return Err(VulkanImageTransitionError::ForeignQueueFamilyUnsupported);
        }

        let barrier = [vk::ImageMemoryBarrier::default()
            .src_access_mask(old_state.access())
            .dst_access_mask(vk::AccessFlags::empty())
            .old_layout(old_state.layout())
            .new_layout(vk::ImageLayout::GENERAL)
            .src_queue_family_index(device.graphics_queue_family())
            .dst_queue_family_index(vk::QUEUE_FAMILY_FOREIGN_EXT)
            .image(image)
            .subresource_range(Self::color_subresource_range())];

        unsafe {
            device.handle().cmd_pipeline_barrier(
                command_buffer,
                old_state.stage(),
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &barrier,
            );
        }
        Ok(())
    }

    /// Acquires a DMA-BUF-backed image back from KMS/foreign ownership.
    ///
    /// Call only after the KMS page-flip/release event confirms that no active
    /// plane references the image. The image is expected to be in GENERAL
    /// layout while owned by the foreign consumer.
    pub fn record_acquire_from_foreign(
        device: &VulkanLogicalDevice,
        command_buffer: vk::CommandBuffer,
        image: vk::Image,
        new_state: VulkanImageState,
    ) -> Result<(), VulkanImageTransitionError> {
        if !device.supports_foreign_queue_family() {
            return Err(VulkanImageTransitionError::ForeignQueueFamilyUnsupported);
        }

        let barrier = [vk::ImageMemoryBarrier::default()
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(new_state.access())
            .old_layout(vk::ImageLayout::GENERAL)
            .new_layout(new_state.layout())
            .src_queue_family_index(vk::QUEUE_FAMILY_FOREIGN_EXT)
            .dst_queue_family_index(device.graphics_queue_family())
            .image(image)
            .subresource_range(Self::color_subresource_range())];

        unsafe {
            device.handle().cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                new_state.stage(),
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &barrier,
            );
        }
        Ok(())
    }

    fn color_subresource_range() -> vk::ImageSubresourceRange {
        vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1)
    }

    /// Records a color image barrier in the supplied command buffer.
    ///
    /// This helper intentionally does not track image state across submissions.
    /// The caller owns that state machine and must serialize conflicting access.
    pub fn record_color(
        device: &VulkanLogicalDevice,
        command_buffer: vk::CommandBuffer,
        image: vk::Image,
        old_state: VulkanImageState,
        new_state: VulkanImageState,
    ) {
        let barrier = [vk::ImageMemoryBarrier::default()
            .src_access_mask(old_state.access())
            .dst_access_mask(new_state.access())
            .old_layout(old_state.layout())
            .new_layout(new_state.layout())
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(Self::color_subresource_range())];

        unsafe {
            device.handle().cmd_pipeline_barrier(
                command_buffer,
                old_state.stage(),
                new_state.stage(),
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &barrier,
            );
        }
    }
}
