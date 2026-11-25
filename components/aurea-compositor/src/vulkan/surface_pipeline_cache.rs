// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Persistent Vulkan resources for SHM surface composition.

use std::{collections::HashMap, error::Error, fmt, path::PathBuf, sync::Arc};

use smithay::{reexports::ash::vk, reexports::wayland_server::backend::ObjectId};

use crate::{
    desktop::{
        DesktopIconTextureKey, DesktopTheme, WallpaperAsset, WallpaperDecodeService,
        WallpaperDecodeServiceError, WallpaperPixels, desktop_context_menu_asset,
        load_desktop_icon_snapshot,
    },
    render::shm_buffer::ShmBufferSnapshot,
};

use super::{
    VulkanGraphicsPipeline, VulkanGraphicsPipelineError, VulkanGraphicsShaders,
    VulkanLogicalDevice, VulkanSampler, VulkanSamplerError, VulkanShaderError, VulkanShaderModule,
    VulkanShmTexture, VulkanTextureBindingError, VulkanTextureBindingPool,
    VulkanNativeCursorTexture,
    VulkanTextureDescriptorSetLayout, VulkanTextureLayoutError,
    surface_texture_cache::{VulkanSurfaceTextureCache, VulkanSurfaceTextureCacheError},
    desktop_icon_texture_cache::{VulkanDesktopIconTextureCache, VulkanDesktopIconTextureCacheError},
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
    descriptor_layout: VulkanTextureDescriptorSetLayout<'static>,
    sampler: VulkanSampler<'static>,
    texture_cache: VulkanSurfaceTextureCache,
    wallpaper_texture_cache: VulkanWallpaperTextureCache,
    desktop_icon_texture_cache: VulkanDesktopIconTextureCache,
    context_menu_assets: HashMap<DesktopIconTextureKey, Option<ShmBufferSnapshot>>,
    wallpaper_decode_service: WallpaperDecodeService,
    native_cursor_texture: VulkanNativeCursorTexture,
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

        Ok(Self {
            device,
            pipelines: HashMap::new(),
            wallpaper_pipelines: HashMap::new(),
            descriptor_layout,
            sampler,
            texture_cache,
            wallpaper_texture_cache,
            desktop_icon_texture_cache,
            context_menu_assets: HashMap::new(),
            wallpaper_decode_service,
            native_cursor_texture: VulkanNativeCursorTexture::new(),
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

    /// Creates a wallpaper-specific pipeline with an independent UV push-constant ABI.
    pub(crate) fn ensure_wallpaper_pipeline(
        &mut self,
        format: vk::Format,
    ) -> Result<(), VulkanSurfacePipelineCacheError> {
        if self.wallpaper_pipelines.contains_key(&format) {
            return Ok(());
        }

        let vertex = VulkanShaderModule::from_bytes(
            self.device.as_ref(),
            include_bytes!(env!("AUREA_WALLPAPER_TEXTURE_VERT_SPV")),
        )
        .map_err(VulkanSurfacePipelineCacheError::Shader)?;
        let fragment = VulkanShaderModule::from_bytes(
            self.device.as_ref(),
            include_bytes!(env!("AUREA_WALLPAPER_TEXTURE_FRAG_SPV")),
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
            Err(WallpaperDecodeServiceError::Worker(
                aurea_media::DecodeWorkerError::QueueFull,
            )) => return Ok(()),
            Err(error) => return Err(VulkanWallpaperTextureCacheError::Decode(error.to_string())),
        }

        service.poll().map_err(|error| VulkanWallpaperTextureCacheError::Decode(error.to_string()))?;
        let ready_len = service.ready().map(|decoded| decoded.pixels().byte_len());
        if ready_len.is_some_and(|byte_len| {
            self.wallpaper_texture_cache.can_accept_upload(byte_len)
        }) {
            if let Some(decoded) = service.ready() {
                let key = WallpaperTextureKey {
                    asset_id: decoded.key().asset_id().to_owned(),
                    revision: decoded.key().revision(),
                };
                if self.wallpaper_texture_cache
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

    /// Starts or polls the compositor-owned arrow upload without waiting for the GPU.
    pub(crate) fn ensure_native_cursor_texture(
        &mut self,
    ) -> Result<bool, VulkanSurfacePipelineCacheError> {
        self.native_cursor_texture
            .ensure(Arc::clone(&self.device))
            .map_err(VulkanSurfacePipelineCacheError::NativeCursor)
    }

    pub(crate) fn native_cursor_texture(&self) -> Option<&VulkanShmTexture> {
        self.native_cursor_texture.texture()
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
