// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Reusable full-output Vulkan color adjustment pipeline.

use std::{error::Error, fmt};

use smithay::reexports::ash::vk;

use super::{
    VulkanColorLoad, VulkanDynamicRenderingScope, VulkanGraphicsPipeline, VulkanGraphicsShaders,
    VulkanImageState, VulkanImageTransition, VulkanImageView, VulkanLogicalDevice, VulkanSampler,
    VulkanShaderModule, VulkanTextureBinding, VulkanTextureDescriptorSetLayout,
};

/// Errors raised while creating or recording a full-output color adjustment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VulkanDisplayColorPassError {
    InvalidIntensity,
    UndefinedImageState,
    MissingSampledUsage,
    MissingColorAttachmentUsage,
    IncompatibleImages,
    BindingSourceMismatch,
    BindingLayoutMismatch,
    AliasedImages,
    UnsupportedFormatFeatures,
    Pipeline(String),
    Descriptor(String),
}

impl fmt::Display for VulkanDisplayColorPassError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIntensity => formatter.write_str("Display warmth must be between 0 and 100"),
            Self::UndefinedImageState => formatter.write_str("Display color pass requires defined image states"),
            Self::MissingSampledUsage => formatter.write_str("Display color source lacks sampled-image usage"),
            Self::MissingColorAttachmentUsage => formatter.write_str("Display color target lacks color-attachment usage"),
            Self::IncompatibleImages => formatter.write_str("Display color resources must share a device and compatible dimensions and format"),
            Self::BindingSourceMismatch => formatter.write_str("Display color descriptor binding does not reference the supplied source image view"),
            Self::BindingLayoutMismatch => formatter.write_str("Display color descriptor binding uses a different descriptor-set layout"),
            Self::AliasedImages => formatter.write_str("Display color source and target must be different Vulkan images"),
            Self::UnsupportedFormatFeatures => formatter.write_str("The output format does not support optimal-tiled color attachment and linearly sampled image usage required by the display color pass"),
            Self::Pipeline(error) => write!(formatter, "Display color pipeline initialization failed: {error}"),
            Self::Descriptor(error) => write!(formatter, "Display color descriptor initialization failed: {error}"),
        }
    }
}

impl Error for VulkanDisplayColorPassError {}

/// Owns reusable GPU state for the final display-color transform.
///
/// Create one instance per Vulkan device and scanout format, then reuse it
/// across frames. Per-frame descriptor bindings remain short-lived and are
/// retained until the command buffer has completed.
pub struct VulkanDisplayColorPass<'device> {
    // Keep the pipeline layout's descriptor-set layout alive for this object's
    // entire lifetime. The pipeline itself is dropped before its dependencies.
    pipeline: VulkanGraphicsPipeline<'device>,
    descriptor_layout: VulkanTextureDescriptorSetLayout<'device>,
    sampler: VulkanSampler<'device>,
}

impl<'device> VulkanDisplayColorPass<'device> {
    /// Creates the reusable pipeline for one scanout format.
    pub fn new(
        device: &'device VulkanLogicalDevice,
        target_format: vk::Format,
    ) -> Result<Self, VulkanDisplayColorPassError> {
        // The composition intermediate is an optimal-tiled image, even when
        // the final scanout image uses a DRM modifier. Check its exact format
        // requirements before creating any pipeline resources.
        let format_properties = unsafe {
            device
                .instance()
                .get_physical_device_format_properties(device.physical_device(), target_format)
        };
        let required_features = vk::FormatFeatureFlags::COLOR_ATTACHMENT
            | vk::FormatFeatureFlags::SAMPLED_IMAGE
            | vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR;
        if !format_properties
            .optimal_tiling_features
            .contains(required_features)
        {
            return Err(VulkanDisplayColorPassError::UnsupportedFormatFeatures);
        }

        let descriptor_layout = VulkanTextureDescriptorSetLayout::new(device)
            .map_err(|error| VulkanDisplayColorPassError::Descriptor(error.to_string()))?;
        let sampler = VulkanSampler::linear_clamp(device)
            .map_err(|error| VulkanDisplayColorPassError::Descriptor(error.to_string()))?;

        let vertex = VulkanShaderModule::from_renderer_variants(
            device,
            include_bytes!(env!("AUREA_DISPLAY_COLOR_VERT_SPV")),
            include_bytes!(env!("AUREA_DISPLAY_COLOR_VERT_COMPAT_SPV")),
        )
        .map_err(|error| VulkanDisplayColorPassError::Pipeline(error.to_string()))?;
        let fragment = VulkanShaderModule::from_renderer_variants(
            device,
            include_bytes!(env!("AUREA_DISPLAY_COLOR_FRAG_SPV")),
            include_bytes!(env!("AUREA_DISPLAY_COLOR_FRAG_COMPAT_SPV")),
        )
        .map_err(|error| VulkanDisplayColorPassError::Pipeline(error.to_string()))?;

        let descriptor_layouts = [&descriptor_layout];
        let push_ranges = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size(std::mem::size_of::<f32>() as u32)];
        let pipeline = VulkanGraphicsPipeline::new_with_descriptor_layouts_and_push_constants(
            device,
            VulkanGraphicsShaders {
                vertex: &vertex,
                fragment: &fragment,
            },
            target_format,
            &descriptor_layouts,
            &push_ranges,
        )
        .map_err(|error| VulkanDisplayColorPassError::Pipeline(error.to_string()))?;

        Ok(Self {
            pipeline,
            descriptor_layout,
            sampler,
        })
    }

    /// Creates a reusable pass that retains the Vulkan device independently.
    pub fn new_shared(
        device: std::sync::Arc<VulkanLogicalDevice>,
        target_format: vk::Format,
    ) -> Result<VulkanDisplayColorPass<'static>, VulkanDisplayColorPassError> {
        let device_ref = device.as_ref();
        let format_properties = unsafe {
            device_ref
                .instance()
                .get_physical_device_format_properties(device_ref.physical_device(), target_format)
        };
        let required_features = vk::FormatFeatureFlags::COLOR_ATTACHMENT
            | vk::FormatFeatureFlags::SAMPLED_IMAGE
            | vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR;
        if !format_properties
            .optimal_tiling_features
            .contains(required_features)
        {
            return Err(VulkanDisplayColorPassError::UnsupportedFormatFeatures);
        }

        let descriptor_layout =
            VulkanTextureDescriptorSetLayout::new_shared(std::sync::Arc::clone(&device))
                .map_err(|error| VulkanDisplayColorPassError::Descriptor(error.to_string()))?;
        let sampler = VulkanSampler::linear_clamp_shared(std::sync::Arc::clone(&device))
            .map_err(|error| VulkanDisplayColorPassError::Descriptor(error.to_string()))?;
        let vertex = VulkanShaderModule::from_renderer_variants(
            device_ref,
            include_bytes!(env!("AUREA_DISPLAY_COLOR_VERT_SPV")),
            include_bytes!(env!("AUREA_DISPLAY_COLOR_VERT_COMPAT_SPV")),
        )
        .map_err(|error| VulkanDisplayColorPassError::Pipeline(error.to_string()))?;
        let fragment = VulkanShaderModule::from_renderer_variants(
            device_ref,
            include_bytes!(env!("AUREA_DISPLAY_COLOR_FRAG_SPV")),
            include_bytes!(env!("AUREA_DISPLAY_COLOR_FRAG_COMPAT_SPV")),
        )
        .map_err(|error| VulkanDisplayColorPassError::Pipeline(error.to_string()))?;
        let descriptor_handles = [descriptor_layout.handle()];
        let push_ranges = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size(std::mem::size_of::<f32>() as u32)];
        let pipeline = VulkanGraphicsPipeline::new_shared_with_descriptor_layout_handles(
            std::sync::Arc::clone(&device),
            VulkanGraphicsShaders {
                vertex: &vertex,
                fragment: &fragment,
            },
            target_format,
            &descriptor_handles,
            &push_ranges,
        )
        .map_err(|error| VulkanDisplayColorPassError::Pipeline(error.to_string()))?;

        Ok(VulkanDisplayColorPass {
            pipeline,
            descriptor_layout,
            sampler,
        })
    }

    /// Creates a per-frame sampled-image binding.
    ///
    /// Keep the returned binding alive until the command buffer has completed;
    /// destroying its descriptor pool before GPU completion would invalidate
    /// a descriptor set referenced by recorded commands.
    pub fn bind_source<'resources, 'image, 'image_device>(
        &'resources self,
        source: &'resources VulkanImageView<'image, 'image_device>,
    ) -> Result<
        VulkanTextureBinding<'resources, 'image, 'image_device, 'device>,
        VulkanDisplayColorPassError,
    >
    where
        'device: 'image_device,
        'resources: 'image_device,
        'image: 'image_device,
    {
        let device = source.device();
        if self.descriptor_layout.device_handle() != device.handle().handle()
            || self.sampler.device_handle() != device.handle().handle()
        {
            return Err(VulkanDisplayColorPassError::IncompatibleImages);
        }
        if !source.usage().contains(vk::ImageUsageFlags::SAMPLED) {
            return Err(VulkanDisplayColorPassError::MissingSampledUsage);
        }

        VulkanTextureBinding::new(device, &self.descriptor_layout, source, &self.sampler)
            .map_err(|error| VulkanDisplayColorPassError::Descriptor(error.to_string()))
    }

    /// Records the adjustment from a sampled scene image into a scanout target.
    ///
    /// The source is left in SHADER_READ_ONLY_OPTIMAL. The destination is left
    /// in `final_state`, ready for the caller's next ownership transition.
    pub fn record(
        &self,
        device: &VulkanLogicalDevice,
        command_buffer: vk::CommandBuffer,
        source: &VulkanImageView<'_, '_>,
        target: &VulkanImageView<'_, '_>,
        binding: &VulkanTextureBinding<'_, '_, '_, '_>,
        previous_source_state: VulkanImageState,
        previous_target_state: VulkanImageState,
        final_state: VulkanImageState,
        warmth_percent: u8,
    ) -> Result<(), VulkanDisplayColorPassError> {
        if warmth_percent > 100 {
            return Err(VulkanDisplayColorPassError::InvalidIntensity);
        }
        if previous_source_state == VulkanImageState::Undefined
            || final_state == VulkanImageState::Undefined
        {
            return Err(VulkanDisplayColorPassError::UndefinedImageState);
        }
        if !source.usage().contains(vk::ImageUsageFlags::SAMPLED) {
            return Err(VulkanDisplayColorPassError::MissingSampledUsage);
        }
        if !target
            .usage()
            .contains(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        {
            return Err(VulkanDisplayColorPassError::MissingColorAttachmentUsage);
        }
        if !binding.references_view(source) {
            return Err(VulkanDisplayColorPassError::BindingSourceMismatch);
        }
        if !binding.uses_layout(&self.descriptor_layout) {
            return Err(VulkanDisplayColorPassError::BindingLayoutMismatch);
        }
        if source.image_handle() == target.image_handle() {
            return Err(VulkanDisplayColorPassError::AliasedImages);
        }

        if source.device().handle().handle() != device.handle().handle()
            || target.device().handle().handle() != device.handle().handle()
            || source.width() != target.width()
            || source.height() != target.height()
            || source.format() != target.format()
            || !self.pipeline.is_compatible_with(device, target.format())
        {
            return Err(VulkanDisplayColorPassError::IncompatibleImages);
        }

        VulkanImageTransition::record_color(
            device,
            command_buffer,
            source.image_handle(),
            previous_source_state,
            VulkanImageState::ShaderRead,
        );
        VulkanImageTransition::record_color(
            device,
            command_buffer,
            target.image_handle(),
            previous_target_state,
            VulkanImageState::ColorAttachment,
        );

        let scope = VulkanDynamicRenderingScope::begin_color(
            command_buffer,
            target,
            VulkanColorLoad::DontCare,
        );
        self.pipeline.bind(command_buffer);
        binding.bind(command_buffer, self.pipeline.layout(), 0);
        self.pipeline
            .set_viewport_and_scissor(command_buffer, target.width(), target.height());
        let strength = (warmth_percent as f32) / 100.0;
        self.pipeline.push_constants(
            command_buffer,
            vk::ShaderStageFlags::FRAGMENT,
            0,
            &strength.to_ne_bytes(),
        );
        self.pipeline.draw(command_buffer, 3, 1);
        scope.end();

        VulkanImageTransition::record_color(
            device,
            command_buffer,
            target.image_handle(),
            VulkanImageState::ColorAttachment,
            final_state,
        );

        Ok(())
    }
}


/// Traditional-render-pass display-color pipeline for the Vulkan 1.0 path.
///
/// The pass borrows its persistent render pass so the graphics pipeline cannot
/// outlive the render-pass object it was created against.
pub struct VulkanLegacyDisplayColorPass<'pass, 'device> {
    pipeline: super::VulkanLegacyGraphicsPipeline<'pass, 'device>,
    descriptor_layout: VulkanTextureDescriptorSetLayout<'device>,
    sampler: VulkanSampler<'device>,
}

impl<'pass, 'device> VulkanLegacyDisplayColorPass<'pass, 'device> {
    pub fn new(
        device: &'device VulkanLogicalDevice,
        render_pass: &'pass super::VulkanLegacyRenderPass<'device>,
    ) -> Result<Self, VulkanDisplayColorPassError> {
        if render_pass.device_handle() != device.handle().handle() {
            return Err(VulkanDisplayColorPassError::IncompatibleImages);
        }
        if render_pass.load_op() != vk::AttachmentLoadOp::DONT_CARE {
            return Err(VulkanDisplayColorPassError::IncompatibleImages);
        }

        // Match the modern pass' format contract: the sampled intermediate
        // and color-attachment target both use this optimal-tiled format.
        let format_properties = unsafe {
            device.instance().get_physical_device_format_properties(
                device.physical_device(),
                render_pass.format(),
            )
        };
        let required_features = vk::FormatFeatureFlags::COLOR_ATTACHMENT
            | vk::FormatFeatureFlags::SAMPLED_IMAGE
            | vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR;
        if !format_properties.optimal_tiling_features.contains(required_features) {
            return Err(VulkanDisplayColorPassError::UnsupportedFormatFeatures);
        }

        let descriptor_layout = VulkanTextureDescriptorSetLayout::new(device)
            .map_err(|error| VulkanDisplayColorPassError::Descriptor(error.to_string()))?;
        let sampler = VulkanSampler::linear_clamp(device)
            .map_err(|error| VulkanDisplayColorPassError::Descriptor(error.to_string()))?;
        let vertex = VulkanShaderModule::from_renderer_variants(
            device,
            include_bytes!(env!("AUREA_DISPLAY_COLOR_VERT_SPV")),
            include_bytes!(env!("AUREA_DISPLAY_COLOR_VERT_COMPAT_SPV")),
        )
        .map_err(|error| VulkanDisplayColorPassError::Pipeline(error.to_string()))?;
        let fragment = VulkanShaderModule::from_renderer_variants(
            device,
            include_bytes!(env!("AUREA_DISPLAY_COLOR_FRAG_SPV")),
            include_bytes!(env!("AUREA_DISPLAY_COLOR_FRAG_COMPAT_SPV")),
        )
        .map_err(|error| VulkanDisplayColorPassError::Pipeline(error.to_string()))?;
        let descriptor_layouts = [&descriptor_layout];
        let push_ranges = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size(std::mem::size_of::<f32>() as u32)];
        let pipeline = super::VulkanLegacyGraphicsPipeline::new(
            device,
            render_pass,
            VulkanGraphicsShaders { vertex: &vertex, fragment: &fragment },
            &descriptor_layouts,
            &push_ranges,
        )
        .map_err(|error| VulkanDisplayColorPassError::Pipeline(error.to_string()))?;

        Ok(Self { pipeline, descriptor_layout, sampler })
    }

    pub fn bind_source<'resources, 'image, 'image_device>(
        &'resources self,
        source: &'resources VulkanImageView<'image, 'image_device>,
    ) -> Result<
        VulkanTextureBinding<'resources, 'image, 'image_device, 'device>,
        VulkanDisplayColorPassError,
    >
    where
        'device: 'image_device,
        'resources: 'image_device,
        'image: 'image_device,
    {
        let device = source.device();
        if self.descriptor_layout.device_handle() != device.handle().handle()
            || self.sampler.device_handle() != device.handle().handle()
        {
            return Err(VulkanDisplayColorPassError::IncompatibleImages);
        }
        if !source.usage().contains(vk::ImageUsageFlags::SAMPLED) {
            return Err(VulkanDisplayColorPassError::MissingSampledUsage);
        }
        VulkanTextureBinding::new(device, &self.descriptor_layout, source, &self.sampler)
            .map_err(|error| VulkanDisplayColorPassError::Descriptor(error.to_string()))
    }

    pub fn record(
        &self,
        device: &VulkanLogicalDevice,
        command_buffer: vk::CommandBuffer,
        source: &VulkanImageView<'_, '_>,
        target: &super::VulkanLegacyRenderTarget<'_, '_, '_, '_>,
        binding: &VulkanTextureBinding<'_, '_, '_, '_>,
        previous_source_state: VulkanImageState,
        previous_target_state: VulkanImageState,
        final_state: VulkanImageState,
        warmth_percent: u8,
    ) -> Result<(), VulkanDisplayColorPassError> {
        let view = target.view();
        if warmth_percent > 100 {
            return Err(VulkanDisplayColorPassError::InvalidIntensity);
        }
        if previous_source_state == VulkanImageState::Undefined
            || final_state == VulkanImageState::Undefined
        {
            return Err(VulkanDisplayColorPassError::UndefinedImageState);
        }
        if !source.usage().contains(vk::ImageUsageFlags::SAMPLED) {
            return Err(VulkanDisplayColorPassError::MissingSampledUsage);
        }
        if !view.usage().contains(vk::ImageUsageFlags::COLOR_ATTACHMENT) {
            return Err(VulkanDisplayColorPassError::MissingColorAttachmentUsage);
        }
        if !binding.references_view(source) {
            return Err(VulkanDisplayColorPassError::BindingSourceMismatch);
        }
        if !binding.uses_layout(&self.descriptor_layout) {
            return Err(VulkanDisplayColorPassError::BindingLayoutMismatch);
        }
        if source.image_handle() == view.image_handle() {
            return Err(VulkanDisplayColorPassError::AliasedImages);
        }
        if source.device().handle().handle() != device.handle().handle()
            || view.device().handle().handle() != device.handle().handle()
            || source.width() != view.width()
            || source.height() != view.height()
            || source.format() != view.format()
            || !view.usage().contains(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            || !self.pipeline.is_compatible_with(device, view.format(), target.render_pass_ref())
        {
            return Err(VulkanDisplayColorPassError::IncompatibleImages);
        }

        VulkanImageTransition::record_color(
            device, command_buffer, source.image_handle(),
            previous_source_state, VulkanImageState::ShaderRead,
        );
        VulkanImageTransition::record_color(
            device, command_buffer, view.image_handle(),
            previous_target_state, VulkanImageState::ColorAttachment,
        );

        let scope = target.begin(command_buffer, [0.0; 4]);
        self.pipeline.bind(command_buffer);
        binding.bind(command_buffer, self.pipeline.layout(), 0);
        self.pipeline.set_viewport_and_scissor(command_buffer, view.width(), view.height());
        let strength = (warmth_percent as f32) / 100.0;
        self.pipeline.push_constants(
            command_buffer, vk::ShaderStageFlags::FRAGMENT, 0, &strength.to_ne_bytes(),
        );
        self.pipeline.draw(command_buffer, 3, 1);
        scope.end();

        VulkanImageTransition::record_color(
            device, command_buffer, view.image_handle(),
            VulkanImageState::ColorAttachment, final_state,
        );
        Ok(())
    }
}


/// Shared-ownership display-color pass for long-lived Vulkan 1.0 caches.
///
/// The pipeline retains its compatible render pass, while the descriptor layout
/// and sampler independently retain the logical device. Framebuffer targets and
/// descriptor bindings remain per-frame resources.
pub struct VulkanSharedLegacyDisplayColorPass {
    pipeline: super::VulkanSharedLegacyGraphicsPipeline,
    descriptor_layout: VulkanTextureDescriptorSetLayout<'static>,
    sampler: VulkanSampler<'static>,
}

impl VulkanSharedLegacyDisplayColorPass {
    pub fn new(
        device: std::sync::Arc<VulkanLogicalDevice>,
        render_pass: std::sync::Arc<super::VulkanLegacyRenderPass<'static>>,
    ) -> Result<Self, VulkanDisplayColorPassError> {
        let device_ref = device.as_ref();
        if render_pass.device_handle() != device_ref.handle().handle()
            || render_pass.load_op() != vk::AttachmentLoadOp::DONT_CARE
        {
            return Err(VulkanDisplayColorPassError::IncompatibleImages);
        }
        let format_properties = unsafe {
            device_ref.instance().get_physical_device_format_properties(
                device_ref.physical_device(), render_pass.format(),
            )
        };
        let required_features = vk::FormatFeatureFlags::COLOR_ATTACHMENT
            | vk::FormatFeatureFlags::SAMPLED_IMAGE
            | vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR;
        if !format_properties.optimal_tiling_features.contains(required_features) {
            return Err(VulkanDisplayColorPassError::UnsupportedFormatFeatures);
        }

        let descriptor_layout = VulkanTextureDescriptorSetLayout::new_shared(
            std::sync::Arc::clone(&device),
        ).map_err(|error| VulkanDisplayColorPassError::Descriptor(error.to_string()))?;
        let sampler = VulkanSampler::linear_clamp_shared(
            std::sync::Arc::clone(&device),
        ).map_err(|error| VulkanDisplayColorPassError::Descriptor(error.to_string()))?;
        let vertex = VulkanShaderModule::from_renderer_variants(
            device_ref,
            include_bytes!(env!("AUREA_DISPLAY_COLOR_VERT_SPV")),
            include_bytes!(env!("AUREA_DISPLAY_COLOR_VERT_COMPAT_SPV")),
        ).map_err(|error| VulkanDisplayColorPassError::Pipeline(error.to_string()))?;
        let fragment = VulkanShaderModule::from_renderer_variants(
            device_ref,
            include_bytes!(env!("AUREA_DISPLAY_COLOR_FRAG_SPV")),
            include_bytes!(env!("AUREA_DISPLAY_COLOR_FRAG_COMPAT_SPV")),
        ).map_err(|error| VulkanDisplayColorPassError::Pipeline(error.to_string()))?;
        let layouts = [descriptor_layout.handle()];
        let push_ranges = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size(std::mem::size_of::<f32>() as u32)];
        let pipeline = super::VulkanSharedLegacyGraphicsPipeline::new(
            device, render_pass,
            VulkanGraphicsShaders { vertex: &vertex, fragment: &fragment },
            &layouts, &push_ranges,
        ).map_err(|error| VulkanDisplayColorPassError::Pipeline(error.to_string()))?;

        Ok(Self { pipeline, descriptor_layout, sampler })
    }

    pub fn bind_source<'resources, 'image, 'image_device>(
        &'resources self,
        source: &'resources VulkanImageView<'image, 'image_device>,
    ) -> Result<
        VulkanTextureBinding<'resources, 'image, 'image_device, 'static>,
        VulkanDisplayColorPassError,
    >
    where
        'image_device: 'resources,
        'image: 'image_device,
    {
        let device = source.device();
        if self.descriptor_layout.device_handle() != device.handle().handle()
            || self.sampler.device_handle() != device.handle().handle()
        {
            return Err(VulkanDisplayColorPassError::IncompatibleImages);
        }
        if !source.usage().contains(vk::ImageUsageFlags::SAMPLED) {
            return Err(VulkanDisplayColorPassError::MissingSampledUsage);
        }
        VulkanTextureBinding::new(device, &self.descriptor_layout, source, &self.sampler)
            .map_err(|error| VulkanDisplayColorPassError::Descriptor(error.to_string()))
    }

    pub fn record(
        &self,
        device: &VulkanLogicalDevice,
        command_buffer: vk::CommandBuffer,
        source: &VulkanImageView<'_, '_>,
        target: &super::VulkanLegacyRenderTarget<'_, '_, '_, '_>,
        binding: &VulkanTextureBinding<'_, '_, '_, '_>,
        previous_source_state: VulkanImageState,
        previous_target_state: VulkanImageState,
        final_state: VulkanImageState,
        warmth_percent: u8,
    ) -> Result<(), VulkanDisplayColorPassError> {
        let view = target.view();
        if warmth_percent > 100 {
            return Err(VulkanDisplayColorPassError::InvalidIntensity);
        }
        if previous_source_state == VulkanImageState::Undefined
            || final_state == VulkanImageState::Undefined
        {
            return Err(VulkanDisplayColorPassError::UndefinedImageState);
        }
        if !source.usage().contains(vk::ImageUsageFlags::SAMPLED) {
            return Err(VulkanDisplayColorPassError::MissingSampledUsage);
        }
        if !view.usage().contains(vk::ImageUsageFlags::COLOR_ATTACHMENT) {
            return Err(VulkanDisplayColorPassError::MissingColorAttachmentUsage);
        }
        if !binding.references_view(source) {
            return Err(VulkanDisplayColorPassError::BindingSourceMismatch);
        }
        if !binding.uses_layout(&self.descriptor_layout) {
            return Err(VulkanDisplayColorPassError::BindingLayoutMismatch);
        }
        if source.image_handle() == view.image_handle() {
            return Err(VulkanDisplayColorPassError::AliasedImages);
        }
        if source.device().handle().handle() != device.handle().handle()
            || view.device().handle().handle() != device.handle().handle()
            || source.width() != view.width()
            || source.height() != view.height()
            || source.format() != view.format()
            || !self.pipeline.is_compatible_with_target(
                device, view.format(), target.render_pass_ref(),
            )
        {
            return Err(VulkanDisplayColorPassError::IncompatibleImages);
        }

        VulkanImageTransition::record_color(
            device, command_buffer, source.image_handle(),
            previous_source_state, VulkanImageState::ShaderRead,
        );
        VulkanImageTransition::record_color(
            device, command_buffer, view.image_handle(),
            previous_target_state, VulkanImageState::ColorAttachment,
        );
        let scope = target.begin(command_buffer, [0.0; 4]);
        self.pipeline.bind(command_buffer);
        binding.bind(command_buffer, self.pipeline.layout(), 0);
        self.pipeline.set_viewport_and_scissor(command_buffer, view.width(), view.height());
        let strength = (warmth_percent as f32) / 100.0;
        self.pipeline.push_constants(
            command_buffer, vk::ShaderStageFlags::FRAGMENT, 0, &strength.to_ne_bytes(),
        );
        self.pipeline.draw(command_buffer, 3, 1);
        scope.end();
        VulkanImageTransition::record_color(
            device, command_buffer, view.image_handle(),
            VulkanImageState::ColorAttachment, final_state,
        );
        Ok(())
    }
}
