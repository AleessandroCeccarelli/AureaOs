// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Reusable full-output Vulkan color adjustment pipeline.

use std::{error::Error, fmt};

use smithay::reexports::ash::vk;

use super::{
    VulkanColorLoad, VulkanDynamicRenderingScope, VulkanGraphicsPipeline,
    VulkanGraphicsShaders, VulkanImageState, VulkanImageTransition, VulkanImageView,
    VulkanLogicalDevice, VulkanSampler, VulkanShaderModule, VulkanTextureBinding,
    VulkanTextureDescriptorSetLayout,
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
            device.instance().get_physical_device_format_properties(
                device.physical_device(),
                target_format,
            )
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

        let vertex = VulkanShaderModule::from_bytes(
            device,
            include_bytes!(env!("AUREA_DISPLAY_COLOR_VERT_SPV")),
        )
        .map_err(|error| VulkanDisplayColorPassError::Pipeline(error.to_string()))?;
        let fragment = VulkanShaderModule::from_bytes(
            device,
            include_bytes!(env!("AUREA_DISPLAY_COLOR_FRAG_SPV")),
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

    /// Creates a per-frame sampled-image binding.
    ///
    /// Keep the returned binding alive until the command buffer has completed;
    /// destroying its descriptor pool before GPU completion would invalidate
    /// a descriptor set referenced by recorded commands.
    pub fn bind_source<'resources, 'image>(
        &'resources self,
        device: &'device VulkanLogicalDevice,
        source: &'resources VulkanImageView<'image, 'device>,
    ) -> Result<VulkanTextureBinding<'resources, 'image, 'device>, VulkanDisplayColorPassError> {
        if source.device().handle().handle() != device.handle().handle() {
            return Err(VulkanDisplayColorPassError::IncompatibleImages);
        }
        if !source.usage().contains(vk::ImageUsageFlags::SAMPLED) {
            return Err(VulkanDisplayColorPassError::MissingSampledUsage);
        }

        VulkanTextureBinding::new(
            device,
            &self.descriptor_layout,
            source,
            &self.sampler,
        )
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
        binding: &VulkanTextureBinding<'_, '_, 'device>,
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
        if !target.usage().contains(vk::ImageUsageFlags::COLOR_ATTACHMENT) {
            return Err(VulkanDisplayColorPassError::MissingColorAttachmentUsage);
        }
        if !binding.references_view(source) {
            return Err(VulkanDisplayColorPassError::BindingSourceMismatch);
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
