// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Readback completion for Vulkan-composited output images.

use smithay::reexports::ash::vk;

use super::{
    BACKGROUND_COLOR, VulkanClearPass, VulkanCommandBuffer, VulkanImageState,
    VulkanImageView, VulkanLogicalDevice, VulkanReadbackBuffer,
    VulkanSurfaceCompositionError,
};

/// Clears an empty scene and copies its RGBA8 result into host-visible memory.
pub(super) fn clear_and_readback(
    device: &VulkanLogicalDevice,
    view: &VulkanImageView<'_, '_>,
    previous_state: VulkanImageState,
    readback: &VulkanReadbackBuffer<'_>,
) -> Result<(), VulkanSurfaceCompositionError> {
    let mut command_buffer = VulkanCommandBuffer::new(device)
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;
    command_buffer
        .begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;

    VulkanClearPass::record_color(
        command_buffer.handle(),
        view,
        previous_state,
        VulkanImageState::General,
        BACKGROUND_COLOR,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;

    readback
        .record_transition_and_copy_rgba8(
            device,
            command_buffer.handle(),
            view,
            VulkanImageState::General,
        )
        .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;

    command_buffer
        .end()
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;
    command_buffer
        .submit_and_wait()
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;

    Ok(())
}
