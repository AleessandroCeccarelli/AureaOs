// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Reusable Vulkan 1.0 render passes and per-image framebuffer targets.

use std::{error::Error, fmt};

use smithay::reexports::ash::vk;

use super::{VulkanColorLoad, VulkanDeviceOwner, VulkanImageView, VulkanLogicalDevice};

/// Errors raised while creating a legacy render pass or framebuffer target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanLegacyRenderPassError {
    Creation(vk::Result),
    DeviceMismatch,
    FormatMismatch,
}

impl fmt::Display for VulkanLegacyRenderPassError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Creation(error) => {
                write!(formatter, "Legacy Vulkan render-pass creation failed: {error:?}")
            }
            Self::DeviceMismatch => formatter
                .write_str("Legacy render pass and image view use different Vulkan devices"),
            Self::FormatMismatch => formatter
                .write_str("Legacy render pass and image view use different color formats"),
        }
    }
}

impl Error for VulkanLegacyRenderPassError {}

/// Owns a reusable traditional render pass for one color format.
///
/// The render pass is independent of any particular image view. Create one
/// framebuffer target per image view with [`Self::create_target`].
pub struct VulkanLegacyRenderPass<'device> {
    device: VulkanDeviceOwner<'device>,
    render_pass: vk::RenderPass,
    format: vk::Format,
}

impl<'device> VulkanLegacyRenderPass<'device> {
    /// Creates a one-attachment, single-subpass render pass for the view format.
    pub fn new(
        device: &'device VulkanLogicalDevice,
        format: vk::Format,
        load: VulkanColorLoad,
    ) -> Result<Self, VulkanLegacyRenderPassError> {
        let load_op = match load {
            VulkanColorLoad::Clear(_) => vk::AttachmentLoadOp::CLEAR,
            VulkanColorLoad::Load => vk::AttachmentLoadOp::LOAD,
            VulkanColorLoad::DontCare => vk::AttachmentLoadOp::DONT_CARE,
        };
        let attachment = [vk::AttachmentDescription::default()
            .format(format)
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
        let create_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachment)
            .subpasses(&subpass)
            .dependencies(&dependencies);
        let render_pass = unsafe { device.handle().create_render_pass(&create_info, None) }
            .map_err(VulkanLegacyRenderPassError::Creation)?;

        Ok(Self {
            device: VulkanDeviceOwner::Borrowed(device),
            render_pass,
            format,
        })
    }

    pub fn handle(&self) -> vk::RenderPass {
        self.render_pass
    }

    pub fn device_handle(&self) -> vk::Device {
        self.device.device().handle().handle()
    }

    pub fn format(&self) -> vk::Format {
        self.format
    }

    /// Creates a framebuffer for one image view without recreating the render pass.
    pub fn create_target<'pass, 'view, 'image>(
        &'pass self,
        view: &'view VulkanImageView<'image, 'device>,
    ) -> Result<VulkanLegacyRenderTarget<'pass, 'view, 'image, 'device>, VulkanLegacyRenderPassError>
    {
        if view.device().handle().handle() != self.device.device().handle().handle() {
            return Err(VulkanLegacyRenderPassError::DeviceMismatch);
        }
        if view.format() != self.format {
            return Err(VulkanLegacyRenderPassError::FormatMismatch);
        }

        let attachments = [view.handle()];
        let create_info = vk::FramebufferCreateInfo::default()
            .render_pass(self.render_pass)
            .attachments(&attachments)
            .width(view.width())
            .height(view.height())
            .layers(1);
        let framebuffer = unsafe {
            self.device
                .device()
                .handle()
                .create_framebuffer(&create_info, None)
        }
        .map_err(VulkanLegacyRenderPassError::Creation)?;

        Ok(VulkanLegacyRenderTarget {
            render_pass: self,
            view,
            framebuffer,
        })
    }
}

impl Drop for VulkanLegacyRenderPass<'_> {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device()
                .handle()
                .destroy_render_pass(self.render_pass, None);
        }
    }
}

/// Owns a framebuffer and borrows the render pass and image view it connects.
///
/// Rust prevents this target from outliving either dependency. The image must
/// be in COLOR_ATTACHMENT_OPTIMAL before a scope begins and remains in that
/// layout afterwards; callers use VulkanImageTransition for barriers.
pub struct VulkanLegacyRenderTarget<'pass, 'view, 'image, 'device> {
    render_pass: &'pass VulkanLegacyRenderPass<'device>,
    view: &'view VulkanImageView<'image, 'device>,
    framebuffer: vk::Framebuffer,
}

impl<'pass, 'view, 'image, 'device>
    VulkanLegacyRenderTarget<'pass, 'view, 'image, 'device>
{
    pub fn render_pass(&self) -> vk::RenderPass {
        self.render_pass.handle()
    }

    pub fn render_pass_ref(&self) -> &'pass VulkanLegacyRenderPass<'device> {
        self.render_pass
    }

    pub fn framebuffer(&self) -> vk::Framebuffer {
        self.framebuffer
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
    ) -> VulkanLegacyRenderPassScope<'target, 'pass, 'view, 'image, 'device> {
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: clear_color,
            },
        }];
        let begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass.handle())
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
pub struct VulkanLegacyRenderPassScope<'target, 'pass, 'view, 'image, 'device> {
    target: &'target VulkanLegacyRenderTarget<'pass, 'view, 'image, 'device>,
    command_buffer: vk::CommandBuffer,
    ended: bool,
}

impl VulkanLegacyRenderPassScope<'_, '_, '_, '_, '_> {
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

impl Drop for VulkanLegacyRenderPassScope<'_, '_, '_, '_, '_> {
    fn drop(&mut self) {
        self.end_recording();
    }
}

impl Drop for VulkanLegacyRenderTarget<'_, '_, '_, '_> {
    fn drop(&mut self) {
        unsafe {
            self.view
                .device()
                .handle()
                .destroy_framebuffer(self.framebuffer, None);
        }
    }
}
