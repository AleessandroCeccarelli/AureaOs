// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! SHM surface composition into a Vulkan DRM scanout image.

use std::{error::Error, fmt};

use smithay::{
    reexports::ash::vk,
    utils::Transform,
};

use std::sync::Arc;

use crate::{
    render::{RenderScene, geometry::surface_geometry, shm_buffer::ShmBufferSnapshot},
    renderer::RenderTarget,
};

use super::{
    VulkanClearPass, VulkanColorLoad, VulkanCommandBuffer, VulkanDrawPass,
    VulkanGraphicsPipeline, VulkanGraphicsShaders, VulkanImageState, VulkanImageTransition,
    VulkanImageView, VulkanLogicalDevice, VulkanSampler, VulkanShaderModule,
    VulkanShmTexture, VulkanTextureBinding, VulkanTextureDescriptorSetLayout,
};

const BACKGROUND_COLOR: [f32; 4] = [0.08, 0.18, 0.32, 1.0];

/// Errors raised while preparing or recording a SHM-composited frame.
#[derive(Debug)]
pub(crate) enum VulkanSurfaceCompositionError {
    Shader(String),
    Pipeline(String),
    Texture(String),
    Descriptor(String),
    Command(String),
    UnsupportedTransform,
    InvalidSurfaceGeometry,
}

impl fmt::Display for VulkanSurfaceCompositionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shader(error) => write!(formatter, "Surface shader initialization failed: {error}"),
            Self::Pipeline(error) => write!(formatter, "Surface pipeline initialization failed: {error}"),
            Self::Texture(error) => write!(formatter, "Wayland SHM texture upload failed: {error}"),
            Self::Descriptor(error) => write!(formatter, "Surface texture descriptor creation failed: {error}"),
            Self::Command(error) => write!(formatter, "Surface composition command failed: {error}"),
            Self::UnsupportedTransform => formatter.write_str("Non-normal Wayland buffer transforms are not supported by the initial SHM compositor"),
            Self::InvalidSurfaceGeometry => formatter.write_str("A visible Wayland surface has invalid physical geometry"),
        }
    }
}

impl Error for VulkanSurfaceCompositionError {}

struct VisibleSurface {
    snapshot: Arc<ShmBufferSnapshot>,
    geometry: crate::render::geometry::SurfaceGeometry,
    scissor: crate::render::geometry::SurfaceScissor,
}

/// Records and submits one complete SHM-composited frame, releasing the
/// scanout image to the foreign KMS queue only after all sampled resources
/// have remained alive through GPU completion.
pub(crate) fn render_and_release(
    device: &VulkanLogicalDevice,
    view: &VulkanImageView<'_, '_>,
    target: &RenderTarget,
    scene: &RenderScene<'_>,
    previous_image_state: VulkanImageState,
    before_submit: impl FnOnce() -> Result<(), String>,
) -> Result<(), VulkanSurfaceCompositionError> {
    let visible = visible_surfaces(scene, target)?;

    let descriptor_layout = VulkanTextureDescriptorSetLayout::new(device)
        .map_err(|error| VulkanSurfaceCompositionError::Descriptor(error.to_string()))?;
    let sampler = VulkanSampler::linear_clamp(device)
        .map_err(|error| VulkanSurfaceCompositionError::Descriptor(error.to_string()))?;

    let vertex = VulkanShaderModule::from_bytes(
        device,
        include_bytes!(env!("AUREA_SURFACE_TEXTURE_VERT_SPV")),
    )
    .map_err(|error| VulkanSurfaceCompositionError::Shader(error.to_string()))?;
    let fragment = VulkanShaderModule::from_bytes(
        device,
        include_bytes!(env!("AUREA_SURFACE_TEXTURE_FRAG_SPV")),
    )
    .map_err(|error| VulkanSurfaceCompositionError::Shader(error.to_string()))?;

    let descriptor_layouts = [&descriptor_layout];
    let pipeline = VulkanGraphicsPipeline::new_with_descriptor_layouts(
        device,
        VulkanGraphicsShaders {
            vertex: &vertex,
            fragment: &fragment,
        },
        view.format(),
        &descriptor_layouts,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Pipeline(error.to_string()))?;

    // The vectors are built in dependency order. All texture images, views,
    // samplers, descriptor pools and layouts stay alive until submit_and_wait.
    let textures = visible
        .iter()
        .map(|surface| {
            VulkanShmTexture::from_snapshot(
                device,
                surface.snapshot.as_ref().clone(),
            )
            .map_err(|error| VulkanSurfaceCompositionError::Texture(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let views = textures
        .iter()
        .map(|texture| {
            texture
                .create_view()
                .map_err(|error| VulkanSurfaceCompositionError::Texture(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let bindings = views
        .iter()
        .map(|texture_view| {
            VulkanTextureBinding::new(device, &descriptor_layout, texture_view, &sampler)
                .map_err(|error| VulkanSurfaceCompositionError::Descriptor(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;

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
        BACKGROUND_COLOR,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;

    for (surface, binding) in visible.iter().zip(bindings.iter()) {
        let geometry = surface.geometry;
        let scissor = surface.scissor;
        let viewport = vk::Viewport {
            x: geometry.x() as f32,
            y: geometry.y() as f32,
            width: geometry.width() as f32,
            height: geometry.height() as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        let scissor = vk::Rect2D {
            offset: vk::Offset2D {
                x: i32::try_from(scissor.x).map_err(|_| {
                    VulkanSurfaceCompositionError::InvalidSurfaceGeometry
                })?,
                y: i32::try_from(scissor.y).map_err(|_| {
                    VulkanSurfaceCompositionError::InvalidSurfaceGeometry
                })?,
            },
            extent: vk::Extent2D {
                width: scissor.width,
                height: scissor.height,
            },
        };

        VulkanDrawPass::record_textured_region(
            command_buffer.handle(),
            view,
            &pipeline,
            binding,
            0,
            VulkanImageState::General,
            VulkanImageState::General,
            VulkanColorLoad::Load,
            viewport,
            scissor,
        )
        .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;
    }

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
    before_submit().map_err(VulkanSurfaceCompositionError::Command)?;
    command_buffer
        .submit_and_wait()
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;

    Ok(())
}

fn visible_surfaces(
    scene: &RenderScene<'_>,
    target: &RenderTarget,
) -> Result<Vec<VisibleSurface>, VulkanSurfaceCompositionError> {
    let mut visible = Vec::new();

    for snapshot in scene.shm_snapshots() {
        let geometry = surface_geometry(
            (snapshot.location.x, snapshot.location.y),
            (snapshot.snapshot.width(), snapshot.snapshot.height()),
            snapshot.buffer_scale,
            snapshot.buffer_transform,
            target.logical_origin(),
            target.scale(),
        )
        .ok_or(VulkanSurfaceCompositionError::InvalidSurfaceGeometry)?;

        let Some(scissor) = geometry.scissor(target.width(), target.height()) else {
            continue;
        };

        if snapshot.buffer_transform != Transform::Normal {
            return Err(VulkanSurfaceCompositionError::UnsupportedTransform);
        }

        visible.push(VisibleSurface {
            snapshot: Arc::clone(&snapshot.snapshot),
            geometry,
            scissor,
        });
    }

    Ok(visible)
}
