// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Explicit Synchronization 2 transitions for color images.

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

    fn stage(self) -> vk::PipelineStageFlags2 {
        match self {
            Self::Undefined => vk::PipelineStageFlags2::NONE,
            Self::ColorAttachment => vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            Self::ShaderRead => vk::PipelineStageFlags2::FRAGMENT_SHADER,
            Self::TransferSource | Self::TransferDestination => vk::PipelineStageFlags2::TRANSFER,
            Self::General => vk::PipelineStageFlags2::ALL_COMMANDS,
        }
    }

    fn access(self) -> vk::AccessFlags2 {
        match self {
            Self::Undefined => vk::AccessFlags2::NONE,
            Self::ColorAttachment => {
                vk::AccessFlags2::COLOR_ATTACHMENT_READ
                    | vk::AccessFlags2::COLOR_ATTACHMENT_WRITE
            }
            Self::ShaderRead => vk::AccessFlags2::SHADER_SAMPLED_READ,
            Self::TransferSource => vk::AccessFlags2::TRANSFER_READ,
            Self::TransferDestination => vk::AccessFlags2::TRANSFER_WRITE,
            Self::General => vk::AccessFlags2::MEMORY_READ | vk::AccessFlags2::MEMORY_WRITE,
        }
    }
}

/// Records one explicit color-image transition using vkCmdPipelineBarrier2.
pub struct VulkanImageTransition;

impl VulkanImageTransition {
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
        let barrier = [vk::ImageMemoryBarrier2::default()
            .src_stage_mask(old_state.stage())
            .src_access_mask(old_state.access())
            .dst_stage_mask(new_state.stage())
            .dst_access_mask(new_state.access())
            .old_layout(old_state.layout())
            .new_layout(new_state.layout())
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            )];

        let dependency = vk::DependencyInfo::default()
            .image_memory_barriers(&barrier);

        unsafe {
            device.handle().cmd_pipeline_barrier2(command_buffer, &dependency);
        }
    }
}
