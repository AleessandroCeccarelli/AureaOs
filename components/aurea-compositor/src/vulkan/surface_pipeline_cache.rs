// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Persistent Vulkan resources for SHM surface composition.

use std::{collections::HashMap, error::Error, fmt, sync::Arc};

use smithay::{reexports::ash::vk, reexports::wayland_server::backend::ObjectId};

use crate::{desktop::WallpaperPixels, render::shm_buffer::ShmBufferSnapshot};

use super::{
    VulkanGraphicsPipeline, VulkanGraphicsPipelineError, VulkanGraphicsShaders,
    VulkanLogicalDevice, VulkanSampler, VulkanSamplerError, VulkanShaderError, VulkanShaderModule,
    VulkanShmTexture, VulkanTextureBindingError, VulkanTextureBindingPool,
    VulkanTextureDescriptorSetLayout, VulkanTextureLayoutError,
    surface_texture_cache::{VulkanSurfaceTextureCache, VulkanSurfaceTextureCacheError},
    wallpaper_texture_cache::{
        VulkanWallpaperTextureCache, VulkanWallpaperTextureCacheError, WallpaperTextureKey,
    },
    VulkanWallpaperTexture,
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
    wallpaper_texture_cache: VulkanWallpaperTextureCache,
    native_cursor_texture: Option<VulkanShmTexture>,
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
        let wallpaper_texture_cache = VulkanWallpaperTextureCache::new(Arc::clone(&device));

        Ok(Self {
            device,
            pipelines: HashMap::new(),
            descriptor_layout,
            sampler,
            texture_cache,
            wallpaper_texture_cache,
            native_cursor_texture: None,
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
        let transform_range = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(std::mem::size_of::<u32>() as u32)];
        let pipeline = VulkanGraphicsPipeline::new_shared_with_descriptor_layout_handles(
            Arc::clone(&self.device),
            VulkanGraphicsShaders {
                vertex: &vertex,
                fragment: &fragment,
            },
            format,
            &descriptor_layouts,
            &transform_range,
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

    /// Submits the selected wallpaper pixels without waiting for GPU completion.
    #[allow(dead_code)] // Wired when the static-image decoder produces validated pixels.
    pub(crate) fn prepare_wallpaper_texture(
        &mut self,
        key: WallpaperTextureKey,
        pixels: WallpaperPixels,
    ) -> Result<(), VulkanWallpaperTextureCacheError> {
        self.wallpaper_texture_cache.prepare(key, pixels)
    }

    /// Polls wallpaper upload fences without blocking the render path.
    pub(crate) fn poll_wallpaper_texture_uploads(
        &mut self,
    ) -> Result<(), VulkanWallpaperTextureCacheError> {
        self.wallpaper_texture_cache.poll()
    }

    /// Returns the selected wallpaper only after its upload fence has completed.
    #[allow(dead_code)] // Consumed by the wallpaper draw pass integration.
    pub(crate) fn wallpaper_texture(&self) -> Option<&VulkanWallpaperTexture> {
        self.wallpaper_texture_cache.texture()
    }

    #[allow(dead_code)] // Called when the desktop wallpaper selection is cleared.
    pub(crate) fn clear_wallpaper_texture(&mut self) {
        self.wallpaper_texture_cache.clear();
    }

    /// Lazily uploads the compositor-owned arrow once per Vulkan device.
    pub(crate) fn ensure_native_cursor_texture(&mut self) -> Result<(), String> {
        if self.native_cursor_texture.is_some() {
            return Ok(());
        }

        let snapshot =
            crate::render::cursor::default_cursor_snapshot().map_err(|error| error.to_string())?;
        let texture = VulkanShmTexture::from_snapshot(Arc::clone(&self.device), snapshot)
            .map_err(|error| error.to_string())?;
        self.native_cursor_texture = Some(texture);
        Ok(())
    }

    pub(crate) fn native_cursor_texture(&self) -> Option<&VulkanShmTexture> {
        self.native_cursor_texture.as_ref()
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
