// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Readback completion for Vulkan-composited output images.

use smithay::reexports::ash::vk;

use super::{
    VulkanClearPass, VulkanCommandBuffer, VulkanDisplayColorPass, VulkanImageState,
    VulkanImageView, VulkanLogicalDevice, VulkanReadbackBuffer, VulkanSurfaceCompositionError,
};

/// Clears an empty scene, applies display warmth, and copies the final RGBA8 image.
pub(super) fn clear_display_and_readback(
    device: &VulkanLogicalDevice,
    target_view: &VulkanImageView<'_, '_>,
    previous_target_state: VulkanImageState,
    pass: &VulkanDisplayColorPass<'_>,
    warmth: u8,
    intermediate_view: &VulkanImageView<'_, '_>,
    previous_intermediate_state: VulkanImageState,
    readback: &VulkanReadbackBuffer<'_>,
    background_color: [f32; 4],
) -> Result<(), VulkanSurfaceCompositionError> {
    let binding = pass
        .bind_source(intermediate_view)
        .map_err(|error| VulkanSurfaceCompositionError::Descriptor(error.to_string()))?;
    let mut command_buffer = VulkanCommandBuffer::new(device)
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;
    command_buffer
        .begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;

    VulkanClearPass::record_color(
        command_buffer.handle(),
        intermediate_view,
        previous_intermediate_state,
        VulkanImageState::General,
        background_color,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;
    pass.record(
        device,
        command_buffer.handle(),
        intermediate_view,
        target_view,
        &binding,
        VulkanImageState::General,
        previous_target_state,
        VulkanImageState::General,
        warmth,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;
    readback
        .record_transition_and_copy_rgba8(
            device,
            command_buffer.handle(),
            target_view,
            VulkanImageState::General,
        )
        .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;

    command_buffer
        .end()
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;
    command_buffer
        .submit_and_wait_with(|| {})
        .map_err(VulkanSurfaceCompositionError::from_queue_submit_error)?;

    Ok(())
}

/// Clears an empty scene and copies its RGBA8 result into host-visible memory.
pub(super) fn clear_and_readback(
    device: &VulkanLogicalDevice,
    view: &VulkanImageView<'_, '_>,
    previous_state: VulkanImageState,
    readback: &VulkanReadbackBuffer<'_>,
    background_color: [f32; 4],
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
        background_color,
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
        .submit_and_wait_with(|| {})
        .map_err(VulkanSurfaceCompositionError::from_queue_submit_error)?;

    Ok(())
}
