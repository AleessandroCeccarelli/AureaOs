// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan 1.3 dynamic-rendering scope recording.

use smithay::reexports::ash::vk;

use super::{VulkanImageView, VulkanLogicalDevice};

/// Load behavior for a color attachment at the start of a rendering scope.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VulkanColorLoad {
    Clear([f32; 4]),
    Load,
    DontCare,
}

/// A balanced Vulkan dynamic-rendering scope.
///
/// The scope borrows its attachment view so the view and underlying image
/// cannot be destroyed while the rendering commands are being recorded.
pub struct VulkanDynamicRenderingScope<'view, 'image, 'device> {
    device: &'device VulkanLogicalDevice,
    command_buffer: vk::CommandBuffer,
    _view: &'view VulkanImageView<'image, 'device>,
    ended: bool,
}

impl<'view, 'image, 'device> VulkanDynamicRenderingScope<'view, 'image, 'device> {
    /// Begins a single-color-attachment dynamic-rendering scope.
    ///
    /// The caller must transition the image into COLOR_ATTACHMENT_OPTIMAL
    /// before this call and transition it to its next use afterward.
    pub fn begin_color(
        device: &'device VulkanLogicalDevice,
        command_buffer: vk::CommandBuffer,
        view: &'view VulkanImageView<'image, 'device>,
        load: VulkanColorLoad,
    ) -> Self {
        let clear_value = match load {
            VulkanColorLoad::Clear(color) => vk::ClearValue {
                color: vk::ClearColorValue { float32: color },
            },
            VulkanColorLoad::Load | VulkanColorLoad::DontCare => vk::ClearValue::default(),
        };

        let load_op = match load {
            VulkanColorLoad::Clear(_) => vk::AttachmentLoadOp::CLEAR,
            VulkanColorLoad::Load => vk::AttachmentLoadOp::LOAD,
            VulkanColorLoad::DontCare => vk::AttachmentLoadOp::DONT_CARE,
        };

        let attachment = [vk::RenderingAttachmentInfo::default()
            .image_view(view.handle())
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(load_op)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(clear_value)];

        let rendering_info = vk::RenderingInfo::default()
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D { width: view.width(), height: view.height() },
            })
            .layer_count(1)
            .color_attachments(&attachment);

        unsafe {
            device.handle().cmd_begin_rendering(command_buffer, &rendering_info);
        }

        Self {
            device,
            command_buffer,
            _view: view,
            ended: false,
        }
    }

    /// Ends the active dynamic-rendering scope.
    pub fn end(mut self) {
        self.end_recording();
    }

    fn end_recording(&mut self) {
        if !self.ended {
            unsafe {
                self.device.handle().cmd_end_rendering(self.command_buffer);
            }
            self.ended = true;
        }
    }
}

impl Drop for VulkanDynamicRenderingScope<'_, '_, '_> {
    fn drop(&mut self) {
        self.end_recording();
    }
}
