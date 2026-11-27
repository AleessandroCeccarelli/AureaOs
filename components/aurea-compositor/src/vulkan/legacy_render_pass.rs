// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan 1.0 render-pass/framebuffer target for the compatibility renderer.

use std::{error::Error, fmt};

use smithay::reexports::ash::vk;

use super::{VulkanColorLoad, VulkanImageView};

/// Errors raised while creating a legacy render-pass target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanLegacyRenderPassError {
    Creation(vk::Result),
    DeviceMismatch,
}

impl fmt::Display for VulkanLegacyRenderPassError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Creation(error) => {
                write!(formatter, "Legacy Vulkan render-pass creation failed: {error:?}")
            }
            Self::DeviceMismatch => formatter
                .write_str("Legacy render target and image view use different Vulkan devices"),
        }
    }
}

impl Error for VulkanLegacyRenderPassError {}

/// Owns a traditional render pass and framebuffer for one color image view.
///
/// The image view is borrowed for the target lifetime. Its image must already
/// be in COLOR_ATTACHMENT_OPTIMAL before a scope begins and remains in that
/// layout after the scope ends; callers use VulkanImageTransition for barriers.
pub struct VulkanLegacyRenderTarget<'view, 'image, 'device> {
    view: &'view VulkanImageView<'image, 'device>,
    render_pass: vk::RenderPass,
    framebuffer: vk::Framebuffer,
}

impl<'view, 'image, 'device> VulkanLegacyRenderTarget<'view, 'image, 'device> {
    /// Creates a one-attachment, single-subpass render pass and framebuffer.
    pub fn new(
        view: &'view VulkanImageView<'image, 'device>,
        load: VulkanColorLoad,
    ) -> Result<Self, VulkanLegacyRenderPassError> {
        let device = view.device().handle();
        let load_op = match load {
            VulkanColorLoad::Clear(_) => vk::AttachmentLoadOp::CLEAR,
            VulkanColorLoad::Load => vk::AttachmentLoadOp::LOAD,
            VulkanColorLoad::DontCare => vk::AttachmentLoadOp::DONT_CARE,
        };
        let attachment = [vk::AttachmentDescription::default()
            .format(view.format())
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(load_op)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];
        let color_reference = [vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];
        let subpass = [vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_reference)];
        let dependencies = [
            vk::SubpassDependency::default()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(
                    vk::AccessFlags::COLOR_ATTACHMENT_READ
                        | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                ),
            vk::SubpassDependency::default()
                .src_subpass(0)
                .dst_subpass(vk::SUBPASS_EXTERNAL)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_stage_mask(vk::PipelineStageFlags::BOTTOM_OF_PIPE)
                .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .dst_access_mask(vk::AccessFlags::empty()),
        ];
        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachment)
            .subpasses(&subpass)
            .dependencies(&dependencies);
        let render_pass = unsafe { device.create_render_pass(&render_pass_info, None) }
            .map_err(VulkanLegacyRenderPassError::Creation)?;

        let attachments = [view.handle()];
        let framebuffer_info = vk::FramebufferCreateInfo::default()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(view.width())
            .height(view.height())
            .layers(1);
        let framebuffer = match unsafe { device.create_framebuffer(&framebuffer_info, None) } {
            Ok(framebuffer) => framebuffer,
            Err(error) => {
                unsafe { device.destroy_render_pass(render_pass, None) };
                return Err(VulkanLegacyRenderPassError::Creation(error));
            }
        };

        Ok(Self {
            view,
            render_pass,
            framebuffer,
        })
    }

    pub fn render_pass(&self) -> vk::RenderPass {
        self.render_pass
    }

    pub fn extent(&self) -> vk::Extent2D {
        vk::Extent2D {
            width: self.view.width(),
            height: self.view.height(),
        }
    }

    pub fn view(&self) -> &VulkanImageView<'image, 'device> {
        self.view
    }

    /// Begins a traditional render-pass scope on the supplied command buffer.
    pub fn begin<'target>(
        &'target self,
        command_buffer: vk::CommandBuffer,
        clear_color: [f32; 4],
    ) -> VulkanLegacyRenderPassScope<'target, 'view, 'image, 'device> {
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: clear_color,
            },
        }];
        let begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.extent(),
            })
            .clear_values(&clear_values);
        unsafe {
            self.view.device().handle().cmd_begin_render_pass(
                command_buffer,
                &begin_info,
                vk::SubpassContents::INLINE,
            );
        }
        VulkanLegacyRenderPassScope {
            target: self,
            command_buffer,
            ended: false,
        }
    }
}

/// RAII guard that balances vkCmdBeginRenderPass with vkCmdEndRenderPass.
pub struct VulkanLegacyRenderPassScope<'target, 'view, 'image, 'device> {
    target: &'target VulkanLegacyRenderTarget<'view, 'image, 'device>,
    command_buffer: vk::CommandBuffer,
    ended: bool,
}

impl VulkanLegacyRenderPassScope<'_, '_, '_, '_> {
    pub fn end(mut self) {
        self.end_recording();
    }

    fn end_recording(&mut self) {
        if !self.ended {
            unsafe {
                self.target
                    .view
                    .device()
                    .handle()
                    .cmd_end_render_pass(self.command_buffer);
            }
            self.ended = true;
        }
    }
}

impl Drop for VulkanLegacyRenderPassScope<'_, '_, '_, '_> {
    fn drop(&mut self) {
        self.end_recording();
    }
}

impl Drop for VulkanLegacyRenderTarget<'_, '_, '_> {
    fn drop(&mut self) {
        let device = self.view.device().handle();
        unsafe {
            device.destroy_framebuffer(self.framebuffer, None);
            device.destroy_render_pass(self.render_pass, None);
        }
    }
}
