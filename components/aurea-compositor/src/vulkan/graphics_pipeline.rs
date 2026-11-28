// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Graphics-pipeline creation for Vulkan 1.3 dynamic rendering.

use std::{error::Error, ffi::CString, fmt, sync::Arc};

use smithay::reexports::ash::vk;

use super::{VulkanDeviceOwner, VulkanLegacyRenderPass, VulkanLogicalDevice, VulkanShaderModule};

/// Shader stages used to build the initial compositor graphics pipeline.
pub struct VulkanGraphicsShaders<'shader, 'device> {
    pub vertex: &'shader VulkanShaderModule<'device>,
    pub fragment: &'shader VulkanShaderModule<'device>,
}

/// Errors returned while creating a graphics pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanGraphicsPipelineError {
    DeviceMismatch,
    Creation(vk::Result),
    InvalidPipelineCount,
    FormatMismatch,
}

impl fmt::Display for VulkanGraphicsPipelineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeviceMismatch => formatter
                .write_str("Shader modules and graphics pipeline must use the same Vulkan device"),
            Self::Creation(error) => write!(
                formatter,
                "Vulkan graphics-pipeline creation failed: {error:?}"
            ),
            Self::InvalidPipelineCount => {
                formatter.write_str("Vulkan returned no graphics pipeline")
            }
            Self::FormatMismatch => formatter
                .write_str("Legacy render pass and graphics pipeline use different color formats"),
        }
    }
}

impl Error for VulkanGraphicsPipelineError {}

/// Owns a pipeline layout and one dynamic-rendering-compatible graphics pipeline.
pub struct VulkanGraphicsPipeline<'device> {
    device: VulkanDeviceOwner<'device>,
    layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    color_format: vk::Format,
    render_pass: Option<vk::RenderPass>,
}

/// A traditional graphics pipeline that borrows its compatible render pass.
///
/// The borrow guarantees that the render pass cannot be destroyed while the
/// Vulkan graphics pipeline still refers to it.
pub struct VulkanLegacyGraphicsPipeline<'pass, 'device> {
    pipeline: VulkanGraphicsPipeline<'device>,
    _render_pass: &'pass VulkanLegacyRenderPass<'device>,
}

impl<'pass, 'device> VulkanLegacyGraphicsPipeline<'pass, 'device> {
    pub fn new(
        device: &'device VulkanLogicalDevice,
        render_pass: &'pass VulkanLegacyRenderPass<'device>,
        shaders: VulkanGraphicsShaders<'_, 'device>,
        descriptor_layouts: &[&super::VulkanTextureDescriptorSetLayout<'device>],
        push_constant_ranges: &[vk::PushConstantRange],
    ) -> Result<Self, VulkanGraphicsPipelineError> {
        if render_pass.device_handle() != device.handle().handle() {
            return Err(VulkanGraphicsPipelineError::DeviceMismatch);
        }
        if descriptor_layouts
            .iter()
            .any(|layout| layout.device_handle() != device.handle().handle())
        {
            return Err(VulkanGraphicsPipelineError::DeviceMismatch);
        }
        let pipeline = VulkanGraphicsPipeline::new_with_render_pass(
            device,
            shaders,
            render_pass.format(),
            render_pass.handle(),
            descriptor_layouts,
            push_constant_ranges,
        )?;
        Ok(Self {
            pipeline,
            _render_pass: render_pass,
        })
    }

    pub fn handle(&self) -> vk::Pipeline {
        self.pipeline.handle()
    }

    pub fn layout(&self) -> vk::PipelineLayout {
        self.pipeline.layout()
    }

    pub fn bind(&self, command_buffer: vk::CommandBuffer) {
        self.pipeline.bind(command_buffer);
    }

    pub fn push_constants(
        &self,
        command_buffer: vk::CommandBuffer,
        stages: vk::ShaderStageFlags,
        offset: u32,
        data: &[u8],
    ) {
        self.pipeline
            .push_constants(command_buffer, stages, offset, data);
    }

    pub fn draw(&self, command_buffer: vk::CommandBuffer, vertex_count: u32, instance_count: u32) {
        self.pipeline
            .draw(command_buffer, vertex_count, instance_count);
    }

    pub fn set_viewport_and_scissor(
        &self,
        command_buffer: vk::CommandBuffer,
        width: u32,
        height: u32,
    ) {
        self.pipeline
            .set_viewport_and_scissor(command_buffer, width, height);
    }

    pub fn set_viewport_and_scissor_region(
        &self,
        command_buffer: vk::CommandBuffer,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
    ) {
        self.pipeline
            .set_viewport_and_scissor_region(command_buffer, viewport, scissor);
    }

    pub fn is_compatible_with(
        &self,
        device: &VulkanLogicalDevice,
        color_format: vk::Format,
        render_pass: &VulkanLegacyRenderPass<'device>,
    ) -> bool {
        self.pipeline
            .is_compatible_with_render_pass(device, color_format, render_pass.handle())
    }
}

/// A traditional graphics pipeline that owns a shared reference to its render pass.
///
/// Unlike the borrowed wrapper, this form can be stored in a long-lived resource
/// cache without making the cache self-referential. The render pass and logical
/// device remain alive for at least as long as the pipeline.
pub struct VulkanSharedLegacyGraphicsPipeline {
    pipeline: VulkanGraphicsPipeline<'static>,
    render_pass: Arc<VulkanLegacyRenderPass<'static>>,
}

impl VulkanSharedLegacyGraphicsPipeline {
    pub fn new(
        device: Arc<VulkanLogicalDevice>,
        render_pass: Arc<VulkanLegacyRenderPass<'static>>,
        shaders: VulkanGraphicsShaders<'_, '_>,
        descriptor_layout_handles: &[vk::DescriptorSetLayout],
        push_constant_ranges: &[vk::PushConstantRange],
    ) -> Result<Self, VulkanGraphicsPipelineError> {
        if render_pass.device_handle() != device.handle().handle() {
            return Err(VulkanGraphicsPipelineError::DeviceMismatch);
        }
        let device_ref = Arc::clone(&device);
        let pipeline = VulkanGraphicsPipeline::create(
            device_ref.as_ref(),
            VulkanDeviceOwner::Shared(device),
            shaders,
            render_pass.format(),
            descriptor_layout_handles,
            push_constant_ranges,
            Some(render_pass.handle()),
        )?;
        Ok(Self {
            pipeline,
            render_pass,
        })
    }

    pub fn handle(&self) -> vk::Pipeline {
        self.pipeline.handle()
    }
    pub fn layout(&self) -> vk::PipelineLayout {
        self.pipeline.layout()
    }
    pub fn bind(&self, command_buffer: vk::CommandBuffer) {
        self.pipeline.bind(command_buffer);
    }
    pub fn push_constants(
        &self,
        command_buffer: vk::CommandBuffer,
        stages: vk::ShaderStageFlags,
        offset: u32,
        data: &[u8],
    ) {
        self.pipeline
            .push_constants(command_buffer, stages, offset, data);
    }
    pub fn draw(&self, command_buffer: vk::CommandBuffer, vertex_count: u32, instance_count: u32) {
        self.pipeline
            .draw(command_buffer, vertex_count, instance_count);
    }
    pub fn set_viewport_and_scissor(
        &self,
        command_buffer: vk::CommandBuffer,
        width: u32,
        height: u32,
    ) {
        self.pipeline
            .set_viewport_and_scissor(command_buffer, width, height);
    }
    pub fn set_viewport_and_scissor_region(
        &self,
        command_buffer: vk::CommandBuffer,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
    ) {
        self.pipeline
            .set_viewport_and_scissor_region(command_buffer, viewport, scissor);
    }
    /// Returns the exact render pass retained by this pipeline.
    ///
    /// Framebuffers used with this pipeline must be created from this render
    /// pass instance, not merely from another pass with the same format/load op.
    pub(crate) fn render_pass(&self) -> &Arc<VulkanLegacyRenderPass<'static>> {
        &self.render_pass
    }

    pub fn is_compatible_with(
        &self,
        device: &VulkanLogicalDevice,
        color_format: vk::Format,
        render_pass: &Arc<VulkanLegacyRenderPass<'static>>,
    ) -> bool {
        Arc::ptr_eq(&self.render_pass, render_pass)
            && self.pipeline.is_compatible_with_render_pass(
                device,
                color_format,
                render_pass.handle(),
            )
    }

    /// Checks compatibility against a framebuffer target's borrowed render pass.
    pub fn is_compatible_with_target(
        &self,
        device: &VulkanLogicalDevice,
        color_format: vk::Format,
        render_pass: &VulkanLegacyRenderPass<'_>,
    ) -> bool {
        self.pipeline
            .is_compatible_with_render_pass(device, color_format, render_pass.handle())
            && self.render_pass.handle() == render_pass.handle()
    }
}

impl<'device> VulkanGraphicsPipeline<'device> {
    /// Creates a minimal graphics pipeline for one color attachment format.
    ///
    /// Vertex input is empty: the vertex shader is expected to generate its
    /// geometry from built-in shader inputs such as VertexIndex.
    pub fn new(
        device: &'device VulkanLogicalDevice,
        shaders: VulkanGraphicsShaders<'_, 'device>,
        color_format: vk::Format,
    ) -> Result<Self, VulkanGraphicsPipelineError> {
        Self::new_with_descriptor_layouts(device, shaders, color_format, &[])
    }

    /// Creates a graphics pipeline with descriptor layouts and push-constant ranges.
    pub fn new_with_descriptor_layouts_and_push_constants(
        device: &'device VulkanLogicalDevice,
        shaders: VulkanGraphicsShaders<'_, 'device>,
        color_format: vk::Format,
        descriptor_layouts: &[&super::VulkanTextureDescriptorSetLayout<'device>],
        push_constant_ranges: &[vk::PushConstantRange],
    ) -> Result<Self, VulkanGraphicsPipelineError> {
        if descriptor_layouts
            .iter()
            .any(|layout| layout.device_handle() != device.handle().handle())
        {
            return Err(VulkanGraphicsPipelineError::DeviceMismatch);
        }
        let handles = descriptor_layouts
            .iter()
            .map(|layout| layout.handle())
            .collect::<Vec<_>>();
        Self::create(
            device,
            VulkanDeviceOwner::Borrowed(device),
            shaders,
            color_format,
            &handles,
            push_constant_ranges,
            None,
        )
    }

    /// Creates a graphics pipeline with the supplied descriptor-set layouts.
    ///
    /// Descriptor layouts must belong to the same Vulkan device. Their
    /// descriptor sets can then be bound before issuing a draw.
    pub fn new_with_descriptor_layouts(
        device: &'device VulkanLogicalDevice,
        shaders: VulkanGraphicsShaders<'_, 'device>,
        color_format: vk::Format,
        descriptor_layouts: &[&super::VulkanTextureDescriptorSetLayout<'device>],
    ) -> Result<Self, VulkanGraphicsPipelineError> {
        if descriptor_layouts
            .iter()
            .any(|layout| layout.device_handle() != device.handle().handle())
        {
            return Err(VulkanGraphicsPipelineError::DeviceMismatch);
        }
        let handles = descriptor_layouts
            .iter()
            .map(|layout| layout.handle())
            .collect::<Vec<_>>();
        Self::create(
            device,
            VulkanDeviceOwner::Borrowed(device),
            shaders,
            color_format,
            &handles,
            &[],
            None,
        )
    }

    /// Creates a pipeline whose device is retained independently of the caller.
    /// Descriptor layout handles are consumed only during Vulkan pipeline-layout creation;
    /// the owner of those layouts must keep them alive for the pipeline's use contract.
    pub fn new_shared_with_descriptor_layout_handles(
        device: Arc<VulkanLogicalDevice>,
        shaders: VulkanGraphicsShaders<'_, '_>,
        color_format: vk::Format,
        descriptor_layout_handles: &[vk::DescriptorSetLayout],
        push_constant_ranges: &[vk::PushConstantRange],
    ) -> Result<VulkanGraphicsPipeline<'static>, VulkanGraphicsPipelineError> {
        let device_ref = Arc::clone(&device);
        Self::create(
            device_ref.as_ref(),
            VulkanDeviceOwner::Shared(device),
            shaders,
            color_format,
            descriptor_layout_handles,
            push_constant_ranges,
            None,
        )
    }

    /// Creates a traditional render-pass-compatible graphics pipeline.
    ///
    /// The render pass must remain alive for the lifetime of this pipeline and
    /// must be compatible with the supplied color format and subpass zero.
    fn new_with_render_pass(
        device: &'device VulkanLogicalDevice,
        shaders: VulkanGraphicsShaders<'_, 'device>,
        color_format: vk::Format,
        render_pass: vk::RenderPass,
        descriptor_layouts: &[&super::VulkanTextureDescriptorSetLayout<'device>],
        push_constant_ranges: &[vk::PushConstantRange],
    ) -> Result<Self, VulkanGraphicsPipelineError> {
        if render_pass == vk::RenderPass::null()
            || descriptor_layouts
                .iter()
                .any(|layout| layout.device_handle() != device.handle().handle())
        {
            return Err(VulkanGraphicsPipelineError::DeviceMismatch);
        }
        let handles = descriptor_layouts
            .iter()
            .map(|layout| layout.handle())
            .collect::<Vec<_>>();
        Self::create(
            device,
            VulkanDeviceOwner::Borrowed(device),
            shaders,
            color_format,
            &handles,
            push_constant_ranges,
            Some(render_pass),
        )
    }

    fn create<'owner>(
        device: &VulkanLogicalDevice,
        owner: VulkanDeviceOwner<'owner>,
        shaders: VulkanGraphicsShaders<'_, '_>,
        color_format: vk::Format,
        descriptor_layout_handles: &[vk::DescriptorSetLayout],
        push_constant_ranges: &[vk::PushConstantRange],
        render_pass: Option<vk::RenderPass>,
    ) -> Result<VulkanGraphicsPipeline<'owner>, VulkanGraphicsPipelineError> {
        if shaders.vertex.device_handle() != device.handle().handle()
            || shaders.fragment.device_handle() != device.handle().handle()
        {
            return Err(VulkanGraphicsPipelineError::DeviceMismatch);
        }

        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(descriptor_layout_handles)
            .push_constant_ranges(push_constant_ranges);
        let layout = unsafe { device.handle().create_pipeline_layout(&layout_info, None) }
            .map_err(VulkanGraphicsPipelineError::Creation)?;

        let entry = CString::new("main").expect("static shader entry point has no NUL");
        let stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(shaders.vertex.handle())
                .name(&entry),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(shaders.fragment.handle())
                .name(&entry),
        ];

        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default();
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);
        let viewport = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);
        let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false)
            .line_width(1.0);
        let multisample = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false);
        let blend_attachment = [vk::PipelineColorBlendAttachmentState::default()
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(vk::ColorComponentFlags::RGBA)];
        let blend = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(&blend_attachment);
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
        let color_formats = [color_format];
        let mut rendering =
            vk::PipelineRenderingCreateInfo::default().color_attachment_formats(&color_formats);

        let mut pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport)
            .rasterization_state(&rasterization)
            .multisample_state(&multisample)
            .color_blend_state(&blend)
            .dynamic_state(&dynamic)
            .layout(layout)
            .render_pass(render_pass.unwrap_or(vk::RenderPass::null()))
            .subpass(0);
        if render_pass.is_none() {
            pipeline_info.push_next(&mut rendering);
        }
        let pipeline_info = [pipeline_info];

        let pipeline = match unsafe {
            device.handle().create_graphics_pipelines(
                vk::PipelineCache::null(),
                &pipeline_info,
                None,
            )
        } {
            Ok(mut pipelines) => match pipelines.pop() {
                Some(pipeline) => pipeline,
                None => {
                    unsafe { device.handle().destroy_pipeline_layout(layout, None) };
                    return Err(VulkanGraphicsPipelineError::InvalidPipelineCount);
                }
            },
            Err((partial, error)) => {
                for pipeline in partial {
                    unsafe { device.handle().destroy_pipeline(pipeline, None) };
                }
                unsafe { device.handle().destroy_pipeline_layout(layout, None) };
                return Err(VulkanGraphicsPipelineError::Creation(error));
            }
        };

        Ok(VulkanGraphicsPipeline {
            device: owner,
            layout,
            pipeline,
            color_format,
            render_pass,
        })
    }

    pub fn handle(&self) -> vk::Pipeline {
        self.pipeline
    }
    pub fn layout(&self) -> vk::PipelineLayout {
        self.layout
    }

    pub(super) fn is_compatible_with(
        &self,
        device: &VulkanLogicalDevice,
        color_format: vk::Format,
    ) -> bool {
        self.device.device().handle().handle() == device.handle().handle()
            && self.color_format == color_format
            && self.render_pass.is_none()
    }

    pub(super) fn is_compatible_with_render_pass(
        &self,
        device: &VulkanLogicalDevice,
        color_format: vk::Format,
        render_pass: vk::RenderPass,
    ) -> bool {
        self.device.device().handle().handle() == device.handle().handle()
            && self.color_format == color_format
            && self.render_pass == Some(render_pass)
    }

    /// Binds this pipeline to a recording graphics command buffer.
    pub fn bind(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device.device().handle().cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );
        }
    }

    /// Writes push-constant data for the graphics stages.
    ///
    /// The supplied byte range must be covered by a range declared when
    /// creating this pipeline.
    pub fn push_constants(
        &self,
        command_buffer: vk::CommandBuffer,
        stages: vk::ShaderStageFlags,
        offset: u32,
        data: &[u8],
    ) {
        unsafe {
            self.device.device().handle().cmd_push_constants(
                command_buffer,
                self.layout,
                stages,
                offset,
                data,
            );
        }
    }

    /// Records a non-indexed draw using this pipeline.
    ///
    /// The caller must be inside a compatible dynamic-rendering scope and
    /// must have configured all required dynamic state.
    pub fn draw(&self, command_buffer: vk::CommandBuffer, vertex_count: u32, instance_count: u32) {
        unsafe {
            self.device.device().handle().cmd_draw(
                command_buffer,
                vertex_count,
                instance_count,
                0,
                0,
            );
        }
    }

    /// Sets the viewport and scissor to cover the specified target extent.
    pub fn set_viewport_and_scissor(
        &self,
        command_buffer: vk::CommandBuffer,
        width: u32,
        height: u32,
    ) {
        self.set_viewport_and_scissor_region(
            command_buffer,
            vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: width as f32,
                height: height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            },
            vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D { width, height },
            },
        );
    }

    /// Sets a surface-sized viewport and an independently clipped scissor.
    ///
    /// The viewport may extend beyond the render target; the scissor must be
    /// clipped to the attachment bounds by the caller.
    pub fn set_viewport_and_scissor_region(
        &self,
        command_buffer: vk::CommandBuffer,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
    ) {
        unsafe {
            self.device
                .device()
                .handle()
                .cmd_set_viewport(command_buffer, 0, &[viewport]);
            self.device
                .device()
                .handle()
                .cmd_set_scissor(command_buffer, 0, &[scissor]);
        }
    }
}

impl Drop for VulkanGraphicsPipeline<'_> {
    fn drop(&mut self) {
        if self.device.device().is_unusable() {
            return;
        }
        unsafe {
            self.device
                .device()
                .handle()
                .destroy_pipeline(self.pipeline, None);
            self.device
                .device()
                .handle()
                .destroy_pipeline_layout(self.layout, None);
        }
    }
}
