// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Persistent Vulkan resources for SHM surface composition.

use std::{collections::HashMap, error::Error, fmt, sync::Arc};

use smithay::{reexports::ash::vk, reexports::wayland_server::backend::ObjectId};

use crate::render::shm_buffer::ShmBufferSnapshot;

use super::{
    VulkanGraphicsPipeline, VulkanGraphicsPipelineError, VulkanGraphicsShaders,
    VulkanLogicalDevice, VulkanSampler, VulkanSamplerError, VulkanShaderError, VulkanShaderModule,
    VulkanShmTexture, VulkanTextureBindingError, VulkanTextureBindingPool,
    VulkanTextureDescriptorSetLayout, VulkanTextureLayoutError,
    surface_texture_cache::{VulkanSurfaceTextureCache, VulkanSurfaceTextureCacheError},
};

/// Errors returned while preparing cached surface-composition resources.
#[derive(Debug)]
pub(crate) enum VulkanSurfacePipelineCacheError {
    DescriptorLayout(VulkanTextureLayoutError),
    Sampler(VulkanSamplerError),
    Shader(VulkanShaderError),
    Pipeline(VulkanGraphicsPipelineError),
}

impl fmt::Display for VulkanSurfacePipelineCacheError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DescriptorLayout(error) => {
                write!(formatter, "Surface descriptor layout: {error}")
            }
            Self::Sampler(error) => write!(formatter, "Surface sampler: {error}"),
            Self::Shader(error) => write!(formatter, "Surface shader: {error}"),
            Self::Pipeline(error) => write!(formatter, "Surface pipeline: {error}"),
        }
    }
}

impl Error for VulkanSurfacePipelineCacheError {}

/// Retains device-wide surface resources and one graphics pipeline per target format.
pub(crate) struct VulkanSurfacePipelineCache {
    device: Arc<VulkanLogicalDevice>,
    pipelines: HashMap<vk::Format, VulkanGraphicsPipeline<'static>>,
    descriptor_layout: VulkanTextureDescriptorSetLayout<'static>,
    sampler: VulkanSampler<'static>,
    texture_cache: VulkanSurfaceTextureCache,
}

impl VulkanSurfacePipelineCache {
    pub(crate) fn new(
        device: Arc<VulkanLogicalDevice>,
    ) -> Result<Self, VulkanSurfacePipelineCacheError> {
        let descriptor_layout = VulkanTextureDescriptorSetLayout::new_shared(Arc::clone(&device))
            .map_err(VulkanSurfacePipelineCacheError::DescriptorLayout)?;
        let sampler = VulkanSampler::linear_clamp_shared(Arc::clone(&device))
            .map_err(VulkanSurfacePipelineCacheError::Sampler)?;
        let texture_cache = VulkanSurfaceTextureCache::new(Arc::clone(&device));

        Ok(Self {
            device,
            pipelines: HashMap::new(),
            descriptor_layout,
            sampler,
            texture_cache,
        })
    }

    /// Creates a format-specific pipeline only on its first use.
    pub(crate) fn ensure_pipeline(
        &mut self,
        format: vk::Format,
    ) -> Result<(), VulkanSurfacePipelineCacheError> {
        if self.pipelines.contains_key(&format) {
            return Ok(());
        }

        let vertex = VulkanShaderModule::from_bytes(
            self.device.as_ref(),
            include_bytes!(env!("AUREA_SURFACE_TEXTURE_VERT_SPV")),
        )
        .map_err(VulkanSurfacePipelineCacheError::Shader)?;
        let fragment = VulkanShaderModule::from_bytes(
            self.device.as_ref(),
            include_bytes!(env!("AUREA_SURFACE_TEXTURE_FRAG_SPV")),
        )
        .map_err(VulkanSurfacePipelineCacheError::Shader)?;
        let descriptor_layouts = [self.descriptor_layout.handle()];
        let pipeline = VulkanGraphicsPipeline::new_shared_with_descriptor_layout_handles(
            Arc::clone(&self.device),
            VulkanGraphicsShaders {
                vertex: &vertex,
                fragment: &fragment,
            },
            format,
            &descriptor_layouts,
            &[],
        )
        .map_err(VulkanSurfacePipelineCacheError::Pipeline)?;

        self.pipelines.insert(format, pipeline);
        Ok(())
    }

    pub(crate) fn prepare_surface_textures(
        &mut self,
        active_surface_ids: &[ObjectId],
        visible_surfaces: &[(ObjectId, u64, Arc<ShmBufferSnapshot>)],
    ) -> Result<(), VulkanSurfaceTextureCacheError> {
        self.texture_cache
            .prepare(active_surface_ids, visible_surfaces)
    }

    pub(crate) fn complete_surface_texture_uploads(
        &mut self,
    ) -> Result<(), VulkanSurfaceTextureCacheError> {
        self.texture_cache.complete_pending_uploads()
    }

    pub(crate) fn surface_texture(&self, surface_id: &ObjectId) -> Option<&VulkanShmTexture> {
        self.texture_cache.texture(surface_id)
    }

    pub(crate) fn pipeline(&self, format: vk::Format) -> Option<&VulkanGraphicsPipeline<'static>> {
        self.pipelines.get(&format)
    }

    pub(crate) fn descriptor_layout(&self) -> &VulkanTextureDescriptorSetLayout<'static> {
        &self.descriptor_layout
    }

    pub(crate) fn sampler(&self) -> &VulkanSampler<'static> {
        &self.sampler
    }

    /// Creates a frame-scoped descriptor pool for the requested surface count.
    pub(crate) fn create_texture_binding_pool(
        &self,
        max_sets: u32,
    ) -> Result<VulkanTextureBindingPool, VulkanTextureBindingError> {
        VulkanTextureBindingPool::new(Arc::clone(&self.device), max_sets)
    }
}
