// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Minimal color-attachment pass used to produce the first renderer frame.

use std::{error::Error, fmt};

use smithay::reexports::ash::vk;

use super::{
    VulkanColorLoad, VulkanDynamicRenderingScope, VulkanImageState, VulkanImageTransition,
    VulkanImageView,
};

/// Invalid requests rejected before any Vulkan command is recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanClearPassError {
    NonFiniteClearColor,
    UndefinedFinalState,
    MissingColorAttachmentUsage,
}

impl fmt::Display for VulkanClearPassError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteClearColor => formatter.write_str(
                "Vulkan clear color components must all be finite",
            ),
            Self::UndefinedFinalState => formatter.write_str(
                "A clear pass cannot leave its image in the undefined state",
            ),
            Self::MissingColorAttachmentUsage => formatter.write_str(
                "The Vulkan image was not created with color-attachment usage",
            ),
        }
    }
}

impl Error for VulkanClearPassError {}

/// Records a clear-only color pass using Vulkan 1.3 dynamic rendering.
///
/// This is the smallest complete rendering operation in the AureaOS renderer:
/// it transitions an image, clears its color attachment, and transitions it
/// to the state requested by the next consumer. It deliberately does not
/// submit the command buffer; frame scheduling and submission remain owned
/// by the caller.
pub struct VulkanClearPass;

impl VulkanClearPass {
    /// Validates a clear request without requiring a Vulkan device.
    pub fn validate(
        clear_color: [f32; 4],
        final_state: VulkanImageState,
    ) -> Result<(), VulkanClearPassError> {
        if !clear_color.iter().all(|component| component.is_finite()) {
            return Err(VulkanClearPassError::NonFiniteClearColor);
        }
        if final_state == VulkanImageState::Undefined {
            return Err(VulkanClearPassError::UndefinedFinalState);
        }
        Ok(())
    }

    /// Records image transition, dynamic-rendering clear, and final transition.
    ///
    /// The caller remains responsible for supplying the true previous image
    /// state and serializing conflicting access to the attachment.
    pub fn record_color(
        command_buffer: vk::CommandBuffer,
        view: &VulkanImageView<'_, '_>,
        previous_state: VulkanImageState,
        final_state: VulkanImageState,
        clear_color: [f32; 4],
    ) -> Result<(), VulkanClearPassError> {
        Self::validate(clear_color, final_state)?;
        if !view.usage().contains(vk::ImageUsageFlags::COLOR_ATTACHMENT) {
            return Err(VulkanClearPassError::MissingColorAttachmentUsage);
        }
        let device = view.device();
        let image = view.image_handle();

        VulkanImageTransition::record_color(
            device,
            command_buffer,
            image,
            previous_state,
            VulkanImageState::ColorAttachment,
        );

        let scope = VulkanDynamicRenderingScope::begin_color(
            command_buffer,
            view,
            VulkanColorLoad::Clear(clear_color),
        );
        scope.end();

        VulkanImageTransition::record_color(
            device,
            command_buffer,
            image,
            VulkanImageState::ColorAttachment,
            final_state,
        );
        Ok(())
    }
}
