// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Persistent Vulkan resources for SHM surface composition.

use std::{collections::HashMap, error::Error, fmt, hash::Hash, path::PathBuf, sync::Arc};

use smithay::{reexports::ash::vk, reexports::wayland_server::backend::ObjectId};

use crate::{
    desktop::{
        DesktopIconTextureKey, DesktopTheme, WallpaperAsset, WallpaperDecodeService,
        WallpaperDecodeServiceError, WallpaperPixels, desktop_context_menu_asset,
        desktop_icon_asset, load_desktop_icon_snapshot,
    },
    render::shm_buffer::ShmBufferSnapshot,
};

use super::{
    VulkanGraphicsPipeline, VulkanGraphicsPipelineError, VulkanGraphicsShaders,
    VulkanLogicalDevice, VulkanNativeCursorTexture, VulkanSampler,
    VulkanSamplerError, VulkanShaderError, VulkanShaderModule, VulkanShmTexture,
    VulkanTextureBindingError, VulkanTextureBindingPool, VulkanTextureDescriptorSetLayout,
    VulkanTextureLayoutError, VulkanWallpaperTexture,
    desktop_icon_texture_cache::{
        VulkanDesktopIconTextureCache, VulkanDesktopIconTextureCacheError,
    },
    surface_texture_cache::{VulkanSurfaceTextureCache, VulkanSurfaceTextureCacheError},
    wallpaper_texture_cache::{
        VulkanWallpaperTextureCache, VulkanWallpaperTextureCacheError, WallpaperTextureKey,
    },
};

#[cfg(feature = "legacy-vulkan-frame")]
use super::{
    VulkanLegacyGraphicsPipeline, VulkanLegacyRenderPass, VulkanLegacyRenderPassError,
    VulkanLegacyRenderPassManager, VulkanLegacyDisplayColorManager,
    VulkanDisplayColorPassError, VulkanSharedLegacyDisplayColorPass,
};

/// Errors returned while preparing cached surface-composition resources.
#[derive(Debug)]
pub(crate) enum VulkanSurfacePipelineCacheError {
    DescriptorLayout(VulkanTextureLayoutError),
    Sampler(VulkanSamplerError),
    Shader(VulkanShaderError),
    Pipeline(VulkanGraphicsPipelineError),
    #[cfg(feature = "legacy-vulkan-frame")]
    LegacyRenderPass(VulkanLegacyRenderPassError),
    #[cfg(feature = "legacy-vulkan-frame")]
    DisplayColor(VulkanDisplayColorPassError),
    WallpaperDecode(WallpaperDecodeServiceError),
    NativeCursor(super::VulkanNativeCursorTextureError),
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
            #[cfg(feature = "legacy-vulkan-frame")]
            Self::LegacyRenderPass(error) => {
                write!(formatter, "Legacy surface render pass: {error}")
            }
            #[cfg(feature = "legacy-vulkan-frame")]
            Self::DisplayColor(error) => write!(formatter, "Legacy display-color pass: {error}"),
            Self::WallpaperDecode(error) => write!(formatter, "Wallpaper decode service: {error}"),
            Self::NativeCursor(error) => write!(formatter, "Native cursor texture: {error}"),
        }
    }
}

impl Error for VulkanSurfacePipelineCacheError {}

/// Retains device-wide surface resources and one graphics pipeline per target format.
pub(crate) struct VulkanSurfacePipelineCache {
    device: Arc<VulkanLogicalDevice>,
    pipelines: HashMap<vk::Format, VulkanGraphicsPipeline<'static>>,
    wallpaper_pipelines: HashMap<vk::Format, VulkanGraphicsPipeline<'static>>,
    cursor_trail_pipelines: HashMap<vk::Format, VulkanGraphicsPipeline<'static>>,
    descriptor_layout: VulkanTextureDescriptorSetLayout<'static>,
    sampler: VulkanSampler<'static>,
    texture_cache: VulkanSurfaceTextureCache,
    wallpaper_texture_cache: VulkanWallpaperTextureCache,
    desktop_icon_texture_cache: VulkanDesktopIconTextureCache,
    context_menu_assets: HashMap<DesktopIconTextureKey, Option<ShmBufferSnapshot>>,
    wallpaper_decode_service: WallpaperDecodeService,
    native_cursor_texture: VulkanNativeCursorTexture,
    #[cfg(feature = "legacy-vulkan-frame")]
    legacy_render_passes: VulkanLegacyRenderPassManager,
    #[cfg(feature = "legacy-vulkan-frame")]
    legacy_display_color: VulkanLegacyDisplayColorManager,
    #[cfg(feature = "legacy-vulkan-frame")]
    legacy_surface_pipelines:
        HashMap<(vk::Format, LegacyPipelineLoad), super::VulkanSharedLegacyGraphicsPipeline>,
    #[cfg(feature = "legacy-vulkan-frame")]
    legacy_wallpaper_pipelines:
        HashMap<(vk::Format, LegacyPipelineLoad), super::VulkanSharedLegacyGraphicsPipeline>,
    #[cfg(feature = "legacy-vulkan-frame")]
    legacy_cursor_trail_pipelines:
        HashMap<(vk::Format, LegacyPipelineLoad), super::VulkanSharedLegacyGraphicsPipeline>,
}

#[cfg(feature = "legacy-vulkan-frame")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum LegacyPipelineLoad {
    Clear,
    Load,
    Discard,
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
        let desktop_icon_texture_cache = VulkanDesktopIconTextureCache::new(Arc::clone(&device));
        let wallpaper_decode_service = WallpaperDecodeService::new()
            .map_err(VulkanSurfacePipelineCacheError::WallpaperDecode)?;

        #[cfg(feature = "legacy-vulkan-frame")]
        let legacy_render_passes = VulkanLegacyRenderPassManager::new(Arc::clone(&device));
        #[cfg(feature = "legacy-vulkan-frame")]
        let legacy_display_color = VulkanLegacyDisplayColorManager::new(Arc::clone(&device));

        Ok(Self {
            device,
            pipelines: HashMap::new(),
            wallpaper_pipelines: HashMap::new(),
            cursor_trail_pipelines: HashMap::new(),
            descriptor_layout,
            sampler,
            texture_cache,
            wallpaper_texture_cache,
            desktop_icon_texture_cache,
            context_menu_assets: HashMap::new(),
            wallpaper_decode_service,
            native_cursor_texture: VulkanNativeCursorTexture::new(),
            #[cfg(feature = "legacy-vulkan-frame")]
            legacy_render_passes,
            #[cfg(feature = "legacy-vulkan-frame")]
            legacy_display_color,
            #[cfg(feature = "legacy-vulkan-frame")]
            legacy_surface_pipelines: HashMap::new(),
            #[cfg(feature = "legacy-vulkan-frame")]
            legacy_wallpaper_pipelines: HashMap::new(),
            #[cfg(feature = "legacy-vulkan-frame")]
            legacy_cursor_trail_pipelines: HashMap::new(),
        })
    }

    /// Returns the persistent legacy render pass for a format and clear load operation.
    #[cfg(feature = "legacy-vulkan-frame")]
    pub(crate) fn legacy_clear_render_pass(
        &mut self,
        format: vk::Format,
    ) -> Result<Arc<super::VulkanLegacyRenderPass<'static>>, VulkanLegacyRenderPassError> {
        self.legacy_render_passes.clear(format)
    }

    /// Returns the persistent legacy render pass for a format when preserving prior contents.
    #[cfg(feature = "legacy-vulkan-frame")]
    pub(crate) fn legacy_load_render_pass(
        &mut self,
        format: vk::Format,
    ) -> Result<Arc<super::VulkanLegacyRenderPass<'static>>, VulkanLegacyRenderPassError> {
        self.legacy_render_passes.load(format)
    }

    /// Returns the persistent legacy render pass for a format when prior contents are discarded.
    #[cfg(feature = "legacy-vulkan-frame")]
    pub(crate) fn legacy_discard_render_pass(
        &mut self,
        format: vk::Format,
    ) -> Result<Arc<super::VulkanLegacyRenderPass<'static>>, VulkanLegacyRenderPassError> {
        self.legacy_render_passes.discard(format)
    }

    /// Returns the cached shared legacy display-color pass for an output format.
    #[cfg(feature = "drm")]
    #[cfg(feature = "legacy-vulkan-frame")]
    pub(crate) fn legacy_display_color_pass(
        &mut self,
        format: vk::Format,
    ) -> Result<Arc<VulkanSharedLegacyDisplayColorPass>, VulkanDisplayColorPassError> {
        self.legacy_display_color.get(format)
    }

    /// Creates a format-specific pipeline only on its first use.
    pub(crate) fn ensure_pipeline(
        &mut self,
        format: vk::Format,
    ) -> Result<(), VulkanSurfacePipelineCacheError> {
        if self.pipelines.contains_key(&format) {
            return Ok(());
        }

        let vertex = VulkanShaderModule::from_renderer_variants(
            self.device.as_ref(),
            include_bytes!(env!("AUREA_SURFACE_TEXTURE_VERT_SPV")),
            include_bytes!(env!("AUREA_SURFACE_TEXTURE_VERT_COMPAT_SPV")),
        )
        .map_err(VulkanSurfacePipelineCacheError::Shader)?;
        let fragment = VulkanShaderModule::from_renderer_variants(
            self.device.as_ref(),
            include_bytes!(env!("AUREA_SURFACE_TEXTURE_FRAG_SPV")),
            include_bytes!(env!("AUREA_SURFACE_TEXTURE_FRAG_COMPAT_SPV")),
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

    /// Creates a traditional-render-pass surface pipeline.
    ///
    /// The returned pipeline borrows the supplied persistent render pass.
    /// Keep both the cache and render pass alive for the pipeline's lifetime.
    #[cfg(feature = "legacy-vulkan-frame")]
    pub(crate) fn create_legacy_surface_pipeline<'cache, 'pass>(
        &'cache self,
        render_pass: &'pass VulkanLegacyRenderPass<'cache>,
    ) -> Result<VulkanLegacyGraphicsPipeline<'pass, 'cache>, VulkanSurfacePipelineCacheError> {
        let vertex = VulkanShaderModule::from_renderer_variants(
            self.device.as_ref(),
            include_bytes!(env!("AUREA_SURFACE_TEXTURE_VERT_SPV")),
            include_bytes!(env!("AUREA_SURFACE_TEXTURE_VERT_COMPAT_SPV")),
        )
        .map_err(VulkanSurfacePipelineCacheError::Shader)?;
        let fragment = VulkanShaderModule::from_renderer_variants(
            self.device.as_ref(),
            include_bytes!(env!("AUREA_SURFACE_TEXTURE_FRAG_SPV")),
            include_bytes!(env!("AUREA_SURFACE_TEXTURE_FRAG_COMPAT_SPV")),
        )
        .map_err(VulkanSurfacePipelineCacheError::Shader)?;
        let descriptor_layouts = [&self.descriptor_layout];
        let transform_range = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(std::mem::size_of::<u32>() as u32)];
        VulkanLegacyGraphicsPipeline::new(
            self.device.as_ref(),
            render_pass,
            VulkanGraphicsShaders {
                vertex: &vertex,
                fragment: &fragment,
            },
            &descriptor_layouts,
            &transform_range,
        )
        .map_err(VulkanSurfacePipelineCacheError::Pipeline)
    }

    /// Creates a traditional-render-pass wallpaper pipeline.
    ///
    /// Its four-float UV transform is independent from the surface transform ABI.
    #[cfg(feature = "legacy-vulkan-frame")]
    pub(crate) fn create_legacy_wallpaper_pipeline<'cache, 'pass>(
        &'cache self,
        render_pass: &'pass VulkanLegacyRenderPass<'cache>,
    ) -> Result<VulkanLegacyGraphicsPipeline<'pass, 'cache>, VulkanSurfacePipelineCacheError> {
        let vertex = VulkanShaderModule::from_renderer_variants(
            self.device.as_ref(),
            include_bytes!(env!("AUREA_WALLPAPER_TEXTURE_VERT_SPV")),
            include_bytes!(env!("AUREA_WALLPAPER_TEXTURE_VERT_COMPAT_SPV")),
        )
        .map_err(VulkanSurfacePipelineCacheError::Shader)?;
        let fragment = VulkanShaderModule::from_renderer_variants(
            self.device.as_ref(),
            include_bytes!(env!("AUREA_WALLPAPER_TEXTURE_FRAG_SPV")),
            include_bytes!(env!("AUREA_WALLPAPER_TEXTURE_FRAG_COMPAT_SPV")),
        )
        .map_err(VulkanSurfacePipelineCacheError::Shader)?;
        let descriptor_layouts = [&self.descriptor_layout];
        let uv_range = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size((4 * std::mem::size_of::<f32>()) as u32)];
        VulkanLegacyGraphicsPipeline::new(
            self.device.as_ref(),
            render_pass,
            VulkanGraphicsShaders {
                vertex: &vertex,
                fragment: &fragment,
            },
            &descriptor_layouts,
            &uv_range,
        )
        .map_err(VulkanSurfacePipelineCacheError::Pipeline)
    }

    /// Creates a traditional-render-pass cursor-trail pipeline.
    #[cfg(feature = "legacy-vulkan-frame")]
    pub(crate) fn create_legacy_cursor_trail_pipeline<'cache, 'pass>(
        &'cache self,
        render_pass: &'pass VulkanLegacyRenderPass<'cache>,
    ) -> Result<VulkanLegacyGraphicsPipeline<'pass, 'cache>, VulkanSurfacePipelineCacheError> {
        let vertex = VulkanShaderModule::from_renderer_variants(
            self.device.as_ref(),
            include_bytes!(env!("AUREA_SURFACE_TEXTURE_VERT_SPV")),
            include_bytes!(env!("AUREA_SURFACE_TEXTURE_VERT_COMPAT_SPV")),
        )
        .map_err(VulkanSurfacePipelineCacheError::Shader)?;
        let fragment = VulkanShaderModule::from_renderer_variants(
            self.device.as_ref(),
            include_bytes!(env!("AUREA_CURSOR_TRAIL_FRAG_SPV")),
            include_bytes!(env!("AUREA_CURSOR_TRAIL_FRAG_COMPAT_SPV")),
        )
        .map_err(VulkanSurfacePipelineCacheError::Shader)?;
        let descriptor_layouts = [&self.descriptor_layout];
        let trail_range = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size((2 * std::mem::size_of::<u32>()) as u32)];
        VulkanLegacyGraphicsPipeline::new(
            self.device.as_ref(),
            render_pass,
            VulkanGraphicsShaders {
                vertex: &vertex,
                fragment: &fragment,
            },
            &descriptor_layouts,
            &trail_range,
        )
        .map_err(VulkanSurfacePipelineCacheError::Pipeline)
    }

    /// Creates a shared-ownership legacy surface pipeline suitable for a long-lived cache.
    #[cfg(feature = "legacy-vulkan-frame")]
    pub(crate) fn create_shared_legacy_surface_pipeline(
        &self,
        render_pass: Arc<super::VulkanLegacyRenderPass<'static>>,
    ) -> Result<super::VulkanSharedLegacyGraphicsPipeline, VulkanSurfacePipelineCacheError> {
        let vertex = VulkanShaderModule::from_renderer_variants(
            self.device.as_ref(),
            include_bytes!(env!("AUREA_SURFACE_TEXTURE_VERT_SPV")),
            include_bytes!(env!("AUREA_SURFACE_TEXTURE_VERT_COMPAT_SPV")),
        )
        .map_err(VulkanSurfacePipelineCacheError::Shader)?;
        let fragment = VulkanShaderModule::from_renderer_variants(
            self.device.as_ref(),
            include_bytes!(env!("AUREA_SURFACE_TEXTURE_FRAG_SPV")),
            include_bytes!(env!("AUREA_SURFACE_TEXTURE_FRAG_COMPAT_SPV")),
        )
        .map_err(VulkanSurfacePipelineCacheError::Shader)?;
        let layouts = [self.descriptor_layout.handle()];
        let ranges = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(std::mem::size_of::<u32>() as u32)];
        super::VulkanSharedLegacyGraphicsPipeline::new(
            Arc::clone(&self.device),
            render_pass,
            VulkanGraphicsShaders {
                vertex: &vertex,
                fragment: &fragment,
            },
            &layouts,
            &ranges,
        )
        .map_err(VulkanSurfacePipelineCacheError::Pipeline)
    }

    /// Creates a shared-ownership legacy wallpaper pipeline.
    #[cfg(feature = "legacy-vulkan-frame")]
    pub(crate) fn create_shared_legacy_wallpaper_pipeline(
        &self,
        render_pass: Arc<super::VulkanLegacyRenderPass<'static>>,
    ) -> Result<super::VulkanSharedLegacyGraphicsPipeline, VulkanSurfacePipelineCacheError> {
        let vertex = VulkanShaderModule::from_renderer_variants(
            self.device.as_ref(),
            include_bytes!(env!("AUREA_WALLPAPER_TEXTURE_VERT_SPV")),
            include_bytes!(env!("AUREA_WALLPAPER_TEXTURE_VERT_COMPAT_SPV")),
        )
        .map_err(VulkanSurfacePipelineCacheError::Shader)?;
        let fragment = VulkanShaderModule::from_renderer_variants(
            self.device.as_ref(),
            include_bytes!(env!("AUREA_WALLPAPER_TEXTURE_FRAG_SPV")),
            include_bytes!(env!("AUREA_WALLPAPER_TEXTURE_FRAG_COMPAT_SPV")),
        )
        .map_err(VulkanSurfacePipelineCacheError::Shader)?;
        let layouts = [self.descriptor_layout.handle()];
        let ranges = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size((4 * std::mem::size_of::<f32>()) as u32)];
        super::VulkanSharedLegacyGraphicsPipeline::new(
            Arc::clone(&self.device),
            render_pass,
            VulkanGraphicsShaders {
                vertex: &vertex,
                fragment: &fragment,
            },
            &layouts,
            &ranges,
        )
        .map_err(VulkanSurfacePipelineCacheError::Pipeline)
    }

    /// Creates a shared-ownership legacy cursor-trail pipeline.
    #[cfg(feature = "legacy-vulkan-frame")]
    pub(crate) fn create_shared_legacy_cursor_trail_pipeline(
        &self,
        render_pass: Arc<super::VulkanLegacyRenderPass<'static>>,
    ) -> Result<super::VulkanSharedLegacyGraphicsPipeline, VulkanSurfacePipelineCacheError> {
        let vertex = VulkanShaderModule::from_renderer_variants(
            self.device.as_ref(),
            include_bytes!(env!("AUREA_SURFACE_TEXTURE_VERT_SPV")),
            include_bytes!(env!("AUREA_SURFACE_TEXTURE_VERT_COMPAT_SPV")),
        )
        .map_err(VulkanSurfacePipelineCacheError::Shader)?;
        let fragment = VulkanShaderModule::from_renderer_variants(
            self.device.as_ref(),
            include_bytes!(env!("AUREA_CURSOR_TRAIL_FRAG_SPV")),
            include_bytes!(env!("AUREA_CURSOR_TRAIL_FRAG_COMPAT_SPV")),
        )
        .map_err(VulkanSurfacePipelineCacheError::Shader)?;
        let layouts = [self.descriptor_layout.handle()];
        let ranges = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size((2 * std::mem::size_of::<u32>()) as u32)];
        super::VulkanSharedLegacyGraphicsPipeline::new(
            Arc::clone(&self.device),
            render_pass,
            VulkanGraphicsShaders {
                vertex: &vertex,
                fragment: &fragment,
            },
            &layouts,
            &ranges,
        )
        .map_err(VulkanSurfacePipelineCacheError::Pipeline)
    }

    /// Ensures a cached shared-ownership surface pipeline for a format/load-op pair.
    #[cfg(feature = "legacy-vulkan-frame")]
    pub(crate) fn ensure_shared_legacy_surface_pipeline(
        &mut self,
        format: vk::Format,
        load: LegacyPipelineLoad,
    ) -> Result<(), VulkanSurfacePipelineCacheError> {
        let key = (format, load);
        if self.legacy_surface_pipelines.contains_key(&key) {
            return Ok(());
        }
        let render_pass = match load {
            LegacyPipelineLoad::Clear => self.legacy_clear_render_pass(format),
            LegacyPipelineLoad::Load => self.legacy_load_render_pass(format),
            LegacyPipelineLoad::Discard => self.legacy_discard_render_pass(format),
        }
        .map_err(VulkanSurfacePipelineCacheError::LegacyRenderPass)?;
        let pipeline = self.create_shared_legacy_surface_pipeline(render_pass)?;
        self.legacy_surface_pipelines.insert(key, pipeline);
        Ok(())
    }

    #[cfg(feature = "legacy-vulkan-frame")]
    pub(crate) fn ensure_shared_legacy_wallpaper_pipeline(
        &mut self,
        format: vk::Format,
        load: LegacyPipelineLoad,
    ) -> Result<(), VulkanSurfacePipelineCacheError> {
        let key = (format, load);
        if self.legacy_wallpaper_pipelines.contains_key(&key) {
            return Ok(());
        }
        let render_pass = match load {
            LegacyPipelineLoad::Clear => self.legacy_clear_render_pass(format),
            LegacyPipelineLoad::Load => self.legacy_load_render_pass(format),
            LegacyPipelineLoad::Discard => self.legacy_discard_render_pass(format),
        }
        .map_err(VulkanSurfacePipelineCacheError::LegacyRenderPass)?;
        let pipeline = self.create_shared_legacy_wallpaper_pipeline(render_pass)?;
        self.legacy_wallpaper_pipelines.insert(key, pipeline);
        Ok(())
    }

    #[cfg(feature = "legacy-vulkan-frame")]
    pub(crate) fn ensure_shared_legacy_cursor_trail_pipeline(
        &mut self,
        format: vk::Format,
        load: LegacyPipelineLoad,
    ) -> Result<(), VulkanSurfacePipelineCacheError> {
        let key = (format, load);
        if self.legacy_cursor_trail_pipelines.contains_key(&key) {
            return Ok(());
        }
        let render_pass = match load {
            LegacyPipelineLoad::Clear => self.legacy_clear_render_pass(format),
            LegacyPipelineLoad::Load => self.legacy_load_render_pass(format),
            LegacyPipelineLoad::Discard => self.legacy_discard_render_pass(format),
        }
        .map_err(VulkanSurfacePipelineCacheError::LegacyRenderPass)?;
        let pipeline = self.create_shared_legacy_cursor_trail_pipeline(render_pass)?;
        self.legacy_cursor_trail_pipelines.insert(key, pipeline);
        Ok(())
    }

    /// Prepares every legacy pipeline needed by one composited frame.
    ///
    /// The initial clear uses the dedicated legacy clear pass; all subsequent
    /// layers load the existing attachment contents. Keeping this preparation
    /// in one cache operation ensures the frame recorder never has to create
    /// pipelines while command-buffer recording is in progress.
    #[cfg(feature = "legacy-vulkan-frame")]
    pub(crate) fn prepare_shared_legacy_frame_pipelines(
        &mut self,
        format: vk::Format,
        wallpaper: bool,
        cursor_trail: bool,
    ) -> Result<(), VulkanSurfacePipelineCacheError> {
        self.ensure_shared_legacy_surface_pipeline(format, LegacyPipelineLoad::Load)?;
        if wallpaper {
            self.ensure_shared_legacy_wallpaper_pipeline(format, LegacyPipelineLoad::Load)?;
        }
        if cursor_trail {
            self.ensure_shared_legacy_cursor_trail_pipeline(format, LegacyPipelineLoad::Load)?;
        }
        Ok(())
    }

    /// Prepares the persistent resources required before recording one legacy frame.
    ///
    /// The clear and load render passes are materialized first, followed by the
    /// exact textured pipelines required by the selected frame layers. When
    /// display-color processing is enabled, its independent shared pass is also
    /// created here so descriptor/pipeline allocation stays outside recording.
    /// Prepares the common legacy frame resources without requiring DRM display color.
    ///
    /// This is available to both scanout and readback builds. Output-specific
    /// processing is deliberately layered on top by the DRM caller.
    #[cfg(feature = "legacy-vulkan-frame")]
    pub(crate) fn prepare_shared_legacy_frame_resources_without_display_color(
        &mut self,
        format: vk::Format,
        wallpaper: bool,
        cursor_trail: bool,
    ) -> Result<(), VulkanSurfacePipelineCacheError> {
        self.legacy_clear_render_pass(format)
            .map_err(VulkanSurfacePipelineCacheError::LegacyRenderPass)?;
        self.legacy_load_render_pass(format)
            .map_err(VulkanSurfacePipelineCacheError::LegacyRenderPass)?;
        self.prepare_shared_legacy_frame_pipelines(format, wallpaper, cursor_trail)
    }

    /// Prepares shared frame resources and, when requested, the DRM display-color pass.
    #[cfg(feature = "drm")]
    #[cfg(feature = "legacy-vulkan-frame")]
    pub(crate) fn prepare_shared_legacy_frame_resources(
        &mut self,
        format: vk::Format,
        wallpaper: bool,
        cursor_trail: bool,
        display_color: bool,
    ) -> Result<Option<Arc<VulkanSharedLegacyDisplayColorPass>>, VulkanSurfacePipelineCacheError>
    {
        self.prepare_shared_legacy_frame_resources_without_display_color(
            format,
            wallpaper,
            cursor_trail,
        )?;
        if display_color {
            self.legacy_display_color_pass(format)
                .map(Some)
                .map_err(VulkanSurfacePipelineCacheError::DisplayColor)
        } else {
            Ok(None)
        }
    }

    #[cfg(feature = "legacy-vulkan-frame")]
    pub(crate) fn shared_legacy_surface_pipeline(
        &self,
        format: vk::Format,
        load: LegacyPipelineLoad,
    ) -> Option<&super::VulkanSharedLegacyGraphicsPipeline> {
        self.legacy_surface_pipelines.get(&(format, load))
    }

    #[cfg(feature = "legacy-vulkan-frame")]
    pub(crate) fn shared_legacy_wallpaper_pipeline(
        &self,
        format: vk::Format,
        load: LegacyPipelineLoad,
    ) -> Option<&super::VulkanSharedLegacyGraphicsPipeline> {
        self.legacy_wallpaper_pipelines.get(&(format, load))
    }

    #[cfg(feature = "legacy-vulkan-frame")]
    pub(crate) fn shared_legacy_cursor_trail_pipeline(
        &self,
        format: vk::Format,
        load: LegacyPipelineLoad,
    ) -> Option<&super::VulkanSharedLegacyGraphicsPipeline> {
        self.legacy_cursor_trail_pipelines.get(&(format, load))
    }

    /// Creates a wallpaper-specific pipeline with an independent UV push-constant ABI.
    pub(crate) fn ensure_wallpaper_pipeline(
        &mut self,
        format: vk::Format,
    ) -> Result<(), VulkanSurfacePipelineCacheError> {
        if self.wallpaper_pipelines.contains_key(&format) {
            return Ok(());
        }

        let vertex = VulkanShaderModule::from_renderer_variants(
            self.device.as_ref(),
            include_bytes!(env!("AUREA_WALLPAPER_TEXTURE_VERT_SPV")),
            include_bytes!(env!("AUREA_WALLPAPER_TEXTURE_VERT_COMPAT_SPV")),
        )
        .map_err(VulkanSurfacePipelineCacheError::Shader)?;
        let fragment = VulkanShaderModule::from_renderer_variants(
            self.device.as_ref(),
            include_bytes!(env!("AUREA_WALLPAPER_TEXTURE_FRAG_SPV")),
            include_bytes!(env!("AUREA_WALLPAPER_TEXTURE_FRAG_COMPAT_SPV")),
        )
        .map_err(VulkanSurfacePipelineCacheError::Shader)?;
        let descriptor_layouts = [self.descriptor_layout.handle()];
        let uv_range = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size((4 * std::mem::size_of::<f32>()) as u32)];
        let pipeline = VulkanGraphicsPipeline::new_shared_with_descriptor_layout_handles(
            Arc::clone(&self.device),
            VulkanGraphicsShaders {
                vertex: &vertex,
                fragment: &fragment,
            },
            format,
            &descriptor_layouts,
            &uv_range,
        )
        .map_err(VulkanSurfacePipelineCacheError::Pipeline)?;

        self.wallpaper_pipelines.insert(format, pipeline);
        Ok(())
    }

    pub(crate) fn wallpaper_pipeline(
        &self,
        format: vk::Format,
    ) -> Option<&VulkanGraphicsPipeline<'static>> {
        self.wallpaper_pipelines.get(&format)
    }

    /// Creates an alpha-faded cursor trail pipeline with fragment-stage opacity.
    pub(crate) fn ensure_cursor_trail_pipeline(
        &mut self,
        format: vk::Format,
    ) -> Result<(), VulkanSurfacePipelineCacheError> {
        if self.cursor_trail_pipelines.contains_key(&format) {
            return Ok(());
        }

        let vertex = VulkanShaderModule::from_bytes(
            self.device.as_ref(),
            include_bytes!(env!("AUREA_SURFACE_TEXTURE_VERT_SPV")),
        )
        .map_err(VulkanSurfacePipelineCacheError::Shader)?;
        let fragment = VulkanShaderModule::from_renderer_variants(
            self.device.as_ref(),
            include_bytes!(env!("AUREA_CURSOR_TRAIL_FRAG_SPV")),
            include_bytes!(env!("AUREA_CURSOR_TRAIL_FRAG_COMPAT_SPV")),
        )
        .map_err(VulkanSurfacePipelineCacheError::Shader)?;
        let descriptor_layouts = [self.descriptor_layout.handle()];
        let trail_range = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size((2 * std::mem::size_of::<u32>()) as u32)];
        let pipeline = VulkanGraphicsPipeline::new_shared_with_descriptor_layout_handles(
            Arc::clone(&self.device),
            VulkanGraphicsShaders {
                vertex: &vertex,
                fragment: &fragment,
            },
            format,
            &descriptor_layouts,
            &trail_range,
        )
        .map_err(VulkanSurfacePipelineCacheError::Pipeline)?;

        self.cursor_trail_pipelines.insert(format, pipeline);
        Ok(())
    }

    pub(crate) fn cursor_trail_pipeline(
        &self,
        format: vk::Format,
    ) -> Option<&VulkanGraphicsPipeline<'static>> {
        self.cursor_trail_pipelines.get(&format)
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

    /// Prepares compositor-owned desktop artwork without using Wayland surface IDs.
    pub(crate) fn prepare_desktop_icon_textures(
        &mut self,
        requested: &[(DesktopIconTextureKey, ShmBufferSnapshot)],
    ) -> Result<(), VulkanDesktopIconTextureCacheError> {
        self.desktop_icon_texture_cache.prepare(requested)
    }

    /// Polls native desktop artwork uploads without blocking the render path.
    pub(crate) fn poll_desktop_icon_texture_uploads(
        &mut self,
    ) -> Result<(), VulkanDesktopIconTextureCacheError> {
        self.desktop_icon_texture_cache.poll()
    }

    /// Completes icon uploads before descriptor bindings borrow their image views.
    pub(crate) fn complete_desktop_icon_texture_uploads(
        &mut self,
    ) -> Result<(), VulkanDesktopIconTextureCacheError> {
        self.desktop_icon_texture_cache.complete_pending_uploads()
    }

    pub(crate) fn desktop_icon_texture(
        &self,
        key: DesktopIconTextureKey,
    ) -> Option<&VulkanShmTexture> {
        self.desktop_icon_texture_cache.texture(key)
    }

    /// Loads the selected context-menu artwork once and prepares its cached GPU texture.
    ///
    /// The asset root is configurable for development packages; installed systems
    /// default to the Human Interfaces payload beneath /usr/share/aureaos.
    pub(crate) fn prepare_context_menu_texture(
        &mut self,
        theme: DesktopTheme,
    ) -> Result<Option<DesktopIconTextureKey>, String> {
        let descriptor = desktop_context_menu_asset(theme);
        let key = descriptor.key;
        if !self.context_menu_assets.contains_key(&key) {
            let asset_root = std::env::var_os("AUREA_HUMAN_INTERFACES_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/usr/share/aureaos/human-interfaces"));
            let loaded = load_desktop_icon_snapshot(&asset_root, &descriptor);
            let snapshot = match loaded {
                Ok((_, snapshot)) => Some(snapshot),
                Err(error) => {
                    eprintln!("AureaOS: context-menu artwork unavailable: {error}");
                    None
                }
            };
            self.context_menu_assets.insert(key, snapshot);
        }

        let Some(snapshot) = self.context_menu_assets.get(&key).and_then(Option::as_ref) else {
            return Ok(None);
        };
        if self.desktop_icon_texture_cache.texture(key).is_none() {
            self.desktop_icon_texture_cache
                .prepare(&[(key, snapshot.clone())])
                .map_err(|error| error.to_string())?;
        }
        Ok(Some(key))
    }

    /// Loads and prepares the canonical artwork for one desktop item.
    ///
    /// Decoded RGBA snapshots are retained by texture key, so filesystem access
    /// occurs only when an artwork variant is first requested. The key includes
    /// theme, Trash state and raster density through the asset descriptor.
    pub(crate) fn prepare_desktop_item_texture(
        &mut self,
        item: &aurea_desktop_api::desktop::DesktopItem,
        theme: DesktopTheme,
        trash_state: aurea_desktop_api::desktop::DesktopTrashState,
        output_scale: f64,
    ) -> Result<Option<DesktopIconTextureKey>, String> {
        let Some(descriptor) = desktop_icon_asset(item, theme, trash_state, output_scale) else {
            return Ok(None);
        };
        let key = descriptor.key;
        if !self.context_menu_assets.contains_key(&key) {
            let asset_root = std::env::var_os("AUREA_HUMAN_INTERFACES_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/usr/share/aureaos/human-interfaces"));
            let snapshot = match load_desktop_icon_snapshot(&asset_root, &descriptor) {
                Ok((_, snapshot)) => Some(snapshot),
                Err(error) => {
                    eprintln!("AureaOS: desktop item artwork unavailable: {error}");
                    None
                }
            };
            self.context_menu_assets.insert(key, snapshot);
        }
        let Some(snapshot) = self.context_menu_assets.get(&key).and_then(Option::as_ref) else {
            return Ok(None);
        };
        if self.desktop_icon_texture_cache.texture(key).is_none() {
            self.desktop_icon_texture_cache
                .prepare(&[(key, snapshot.clone())])
                .map_err(|error| error.to_string())?;
        }
        Ok(Some(key))
    }

    /// Advances background decoding and transfers completed pixels to the GPU upload cache.
    ///
    /// The media worker never blocks this frame path. Queue saturation is transient:
    /// the same selection is retried on the next frame.
    pub(crate) fn prepare_wallpaper_decode(
        &mut self,
        asset: Option<WallpaperAsset>,
    ) -> Result<(), VulkanWallpaperTextureCacheError> {
        let Some(asset) = asset else {
            self.wallpaper_texture_cache.clear();
            self.wallpaper_decode_service
                .select(None)
                .map_err(|error| VulkanWallpaperTextureCacheError::Decode(error.to_string()))?;
            return Ok(());
        };

        let service = &mut self.wallpaper_decode_service;
        match service.select(Some(asset)) {
            Ok(_) => {}
            Err(WallpaperDecodeServiceError::Worker(aurea_media::DecodeWorkerError::QueueFull)) => {
                return Ok(());
            }
            Err(error) => return Err(VulkanWallpaperTextureCacheError::Decode(error.to_string())),
        }

        service
            .poll()
            .map_err(|error| VulkanWallpaperTextureCacheError::Decode(error.to_string()))?;
        let ready_len = service.ready().map(|decoded| decoded.pixels().byte_len());
        if ready_len
            .is_some_and(|byte_len| self.wallpaper_texture_cache.can_accept_upload(byte_len))
        {
            if let Some(decoded) = service.ready() {
                let key = WallpaperTextureKey {
                    asset_id: decoded.key().asset_id().to_owned(),
                    revision: decoded.key().revision(),
                };
                if self
                    .wallpaper_texture_cache
                    .prepare(key, decoded.pixels())
                    .map_err(|error| error)?
                {
                    let _ = service.take_ready();
                }
            }
        }
        Ok(())
    }

    /// Submits the selected wallpaper pixels without waiting for GPU completion.
    pub(crate) fn prepare_wallpaper_texture(
        &mut self,
        key: WallpaperTextureKey,
        pixels: &WallpaperPixels,
    ) -> Result<(), VulkanWallpaperTextureCacheError> {
        self.wallpaper_texture_cache
            .prepare(key, pixels)
            .map(|_| ())
    }

    /// Polls wallpaper upload fences without blocking the render path.
    pub(crate) fn poll_wallpaper_texture_uploads(
        &mut self,
    ) -> Result<(), VulkanWallpaperTextureCacheError> {
        self.wallpaper_texture_cache.poll()
    }

    /// Returns the selected wallpaper only after its upload fence has completed.
    /// Returns the ready texture only when it belongs to the requested asset.
    ///
    /// This prevents a previous wallpaper from being drawn during the interval
    /// between a desktop selection change and completion of the replacement upload.
    pub(crate) fn wallpaper_texture_for_asset(
        &self,
        asset_id: &str,
    ) -> Option<&VulkanWallpaperTexture> {
        self.wallpaper_texture_cache.texture_for_asset(asset_id)
    }

    #[allow(dead_code)] // Called when the desktop wallpaper selection is cleared.
    pub(crate) fn clear_wallpaper_texture(&mut self) {
        self.wallpaper_texture_cache.clear();
    }

    /// Loads the selected themed cursor and uploads its frames to persistent GPU textures.
    pub(crate) fn ensure_native_cursor_texture(
        &mut self,
        asset: &str,
        theme: DesktopTheme,
        requested_size: u32,
    ) -> Result<bool, VulkanSurfacePipelineCacheError> {
        self.native_cursor_texture
            .ensure(Arc::clone(&self.device), asset, theme, requested_size)
            .map_err(VulkanSurfacePipelineCacheError::NativeCursor)
    }

    /// Returns whether asynchronous desktop assets need another render-frame poll.
    ///
    /// Wallpaper decoding runs on a worker and its GPU upload is fence-backed.
    /// Keep frames flowing until both stages finish so the first wallpaper is
    /// presented even when no client or input event requests another frame.
    pub(crate) fn has_pending_render_resource_work(&self) -> bool {
        self.native_cursor_texture.has_pending_work()
            || self.wallpaper_decode_service.pending_key().is_some()
            || self.wallpaper_texture_cache.has_pending_work()
    }

    pub(crate) fn native_cursor_asset(
        &self,
        asset: &str,
        theme: DesktopTheme,
        requested_size: u32,
    ) -> Option<&super::native_cursor_texture::VulkanNativeCursorAsset> {
        self.native_cursor_texture
            .asset(asset, theme, requested_size)
    }

    pub(crate) fn pipeline(&self, format: vk::Format) -> Option<&VulkanGraphicsPipeline<'static>> {
        self.pipelines.get(&format)
    }

    /// Clones the device owner for frame-scoped external image imports.
    #[cfg(feature = "drm")]
    pub(crate) fn device_owner(&self) -> Arc<VulkanLogicalDevice> {
        Arc::clone(&self.device)
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
