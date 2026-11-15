// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Full-output Vulkan color adjustment pass.

use std::{error::Error, fmt};

use smithay::reexports::ash::vk;

use super::{
    VulkanColorLoad, VulkanDynamicRenderingScope, VulkanGraphicsPipeline,
    VulkanGraphicsShaders, VulkanImageState, VulkanImageTransition, VulkanImageView,
    VulkanLogicalDevice, VulkanSampler, VulkanShaderModule, VulkanTextureBinding,
    VulkanTextureDescriptorSetLayout,
};

/// Errors raised while recording a full-output color adjustment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VulkanDisplayColorPassError {
    InvalidIntensity,
    UndefinedImageState,
    MissingSampledUsage,
    MissingColorAttachmentUsage,
    IncompatibleImages,
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
            Self::IncompatibleImages => formatter.write_str("Display color source and target must share a device and dimensions"),
            Self::Pipeline(error) => write!(formatter, "Display color pipeline initialization failed: {error}"),
            Self::Descriptor(error) => write!(formatter, "Display color descriptor initialization failed: {error}"),
        }
    }
}

impl Error for VulkanDisplayColorPassError {}

/// Records a full-screen warmth transform after scene composition.
///
/// The initial transform is an SDR-oriented approximation operating on the
/// values supplied by the current compositor path. It is deliberately not
/// advertised as ICC, HDR, or display-native color management.
pub struct VulkanDisplayColorPass;

impl VulkanDisplayColorPass {
    /// Records the adjustment from a sampled scene image into a scanout target.
    ///
    /// The source is left in SHADER_READ_ONLY_OPTIMAL. The destination is left
    /// in `final_state`, ready for the caller's next ownership transition.
    pub fn record(
        device: &VulkanLogicalDevice,
        command_buffer: vk::CommandBuffer,
        source: &VulkanImageView<'_, '_>,
        target: &VulkanImageView<'_, '_>,
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
        if source.device().handle().handle() != device.handle().handle()
            || target.device().handle().handle() != device.handle().handle()
            || source.width() != target.width()
            || source.height() != target.height()
        {
            return Err(VulkanDisplayColorPassError::IncompatibleImages);
        }

        let descriptor_layout = VulkanTextureDescriptorSetLayout::new(device)
            .map_err(|error| VulkanDisplayColorPassError::Descriptor(error.to_string()))?;
        let sampler = VulkanSampler::linear_clamp(device)
            .map_err(|error| VulkanDisplayColorPassError::Descriptor(error.to_string()))?;
        let binding = VulkanTextureBinding::new(device, &descriptor_layout, source, &sampler)
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
            target.format(),
            &descriptor_layouts,
            &push_ranges,
        )
        .map_err(|error| VulkanDisplayColorPassError::Pipeline(error.to_string()))?;

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
        pipeline.bind(command_buffer);
        binding.bind(command_buffer, pipeline.layout(), 0);
        pipeline.set_viewport_and_scissor(command_buffer, target.width(), target.height());
        let strength = (warmth_percent as f32) / 100.0;
        pipeline.push_constants(
            command_buffer,
            vk::ShaderStageFlags::FRAGMENT,
            0,
            &strength.to_ne_bytes(),
        );
        pipeline.draw(command_buffer, 3, 1);
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
