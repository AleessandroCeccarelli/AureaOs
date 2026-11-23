// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Clears scanout targets and releases their images to the foreign KMS queue.

use smithay::reexports::ash::vk;

use super::{
    VulkanClearPass, VulkanCommandBuffer, VulkanDisplayColorPass,
    VulkanImageState, VulkanImageTransition, VulkanImageView, VulkanLogicalDevice,
    VulkanSurfaceCompositionError,
};

pub(super) fn clear_display_and_release(
    device: &VulkanLogicalDevice,
    target_view: &VulkanImageView<'_, '_>,
    previous_target_state: VulkanImageState,
    pass: &VulkanDisplayColorPass<'_>,
    warmth: u8,
    intermediate_view: &VulkanImageView<'_, '_>,
    previous_intermediate_state: VulkanImageState,
    on_submitted: impl FnOnce(),
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
    VulkanImageTransition::record_release_to_foreign(
        device,
        command_buffer.handle(),
        target_view.image_handle(),
        VulkanImageState::General,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;

    command_buffer
        .end()
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;
    command_buffer
        .submit_and_wait_with(on_submitted)
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;

    Ok(())
}

pub(super) fn clear_and_release(
    device: &VulkanLogicalDevice,
    view: &VulkanImageView<'_, '_>,
    previous_image_state: VulkanImageState,
    on_submitted: impl FnOnce(),
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
        previous_image_state,
        VulkanImageState::General,
        background_color,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;
    VulkanImageTransition::record_release_to_foreign(
        device,
        command_buffer.handle(),
        view.image_handle(),
        VulkanImageState::General,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;

    command_buffer
        .end()
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;
    command_buffer
        .submit_and_wait_with(on_submitted)
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;

    Ok(())
}
