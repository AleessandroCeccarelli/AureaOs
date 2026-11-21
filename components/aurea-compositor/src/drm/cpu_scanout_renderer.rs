// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan offscreen rendering with CPU-upload GBM/KMS presentation.

use std::{error::Error, fmt};

use smithay::{
    backend::drm::{PlaneConfig, PlaneState},
    utils::{Buffer, Physical, Rectangle, Size, Transform},
};

use crate::{
    drm::{device::DrmDevice, gbm::GbmDeviceManager, surfaces::ManagedDrmSurface},
    render::RenderScene,
    renderer::{RenderBackend, RenderTarget},
    vulkan::{
        surface_compositor, VulkanBackend, VulkanImageAllocator, VulkanImageRequest,
        VulkanImageState, VulkanImageView, VulkanReadbackBuffer,
    },
};

/// Errors returned by the Vulkan readback compatibility presenter.
#[derive(Debug)]
pub(crate) struct VulkanCpuScanoutError(String);

impl fmt::Display for VulkanCpuScanoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for VulkanCpuScanoutError {}

/// Presents Vulkan-rendered pixels through a CPU-writable GBM buffer.
///
/// This deliberately avoids DMA-BUF export and foreign queue-family ownership.
/// Composes the SHM-backed Wayland scene in Vulkan, reads RGBA8 pixels back,
/// and uploads them into a CPU-writable GBM framebuffer.
pub(crate) struct VulkanCpuOutputBackend<'a> {
    drm_device: &'a DrmDevice,
    gbm_devices: &'a GbmDeviceManager,
    output: &'a mut ManagedDrmSurface,
    vulkan: &'a VulkanBackend,
    surface_cache: &'a mut crate::vulkan::VulkanSurfacePipelineCache,
}

impl<'a> VulkanCpuOutputBackend<'a> {
    pub(crate) fn new(
        drm_device: &'a DrmDevice,
        gbm_devices: &'a GbmDeviceManager,
        output: &'a mut ManagedDrmSurface,
        vulkan: &'a VulkanBackend,
        surface_cache: &'a mut crate::vulkan::VulkanSurfacePipelineCache,
    ) -> Self {
        Self {
            drm_device,
            gbm_devices,
            output,
            vulkan,
            surface_cache,
        }
    }
}

impl RenderBackend for VulkanCpuOutputBackend<'_> {
    type Error = VulkanCpuScanoutError;

    fn render_frame(
        &mut self,
        target: &RenderTarget,
        scene: &RenderScene<'_>,
    ) -> Result<(), Self::Error> {
        if target.output_id() != self.output.output_id() {
            return Err(VulkanCpuScanoutError(
                "Render target does not belong to this DRM surface".into(),
            ));
        }
        if target.width() == 0 || target.height() == 0 {
            return Err(VulkanCpuScanoutError("Output dimensions are zero".into()));
        }

        // Validate KMS coordinate limits before allocating any GBM resources.
        let width = i32::try_from(target.width())
            .map_err(|_| VulkanCpuScanoutError("Output width exceeds KMS coordinates".into()))?;
        let height = i32::try_from(target.height())
            .map_err(|_| VulkanCpuScanoutError("Output height exceeds KMS coordinates".into()))?;

        if !self.drm_device.is_active() {
            return Err(VulkanCpuScanoutError("DRM device is inactive".into()));
        }

        let gbm = self
            .gbm_devices
            .get(self.drm_device.node().map_err(|e| {
                VulkanCpuScanoutError(format!("Unable to identify DRM device: {e}"))
            })?)
            .ok_or_else(|| {
                VulkanCpuScanoutError("GBM device is unavailable for CPU presentation".into())
            })?;

        let drm_fd = self.drm_device.fd();
        let (surface, pool, presenter) = self.output.cpu_parts_mut();
        if surface.is_legacy() {
            return Err(VulkanCpuScanoutError(
                "CPU compatibility presentation currently requires atomic KMS".into(),
            ));
        }
        if pool.pending().is_some() {
            return Ok(());
        }

        let image = VulkanImageAllocator::new(self.vulkan.device())
            .allocate(VulkanImageRequest::render_target(
                target.width(),
                target.height(),
                smithay::reexports::ash::vk::Format::R8G8B8A8_UNORM,
                smithay::reexports::ash::vk::ImageUsageFlags::COLOR_ATTACHMENT
                    | smithay::reexports::ash::vk::ImageUsageFlags::TRANSFER_SRC,
            ))
            .map_err(|error| {
                VulkanCpuScanoutError(format!("Vulkan output image allocation failed: {error}"))
            })?;
        let view = VulkanImageView::color(&image).map_err(|error| {
            VulkanCpuScanoutError(format!("Vulkan output image view failed: {error}"))
        })?;
        let readback = VulkanReadbackBuffer::new(
            self.vulkan.device(),
            target.width(),
            target.height(),
        )
        .map_err(|error| {
            VulkanCpuScanoutError(format!("Vulkan readback allocation failed: {error}"))
        })?;

        surface_compositor::render_and_readback(
            self.vulkan.device(),
            &view,
            target,
            scene,
            self.surface_cache,
            VulkanImageState::Undefined,
            None,
            &readback,
        )
        .map_err(|error| {
            VulkanCpuScanoutError(format!("Vulkan scene composition failed: {error}"))
        })?;
        let pixels = readback.read_rgba8().map_err(|error| {
            VulkanCpuScanoutError(format!("Vulkan output readback failed: {error}"))
        })?;

        let candidate = pool
            .allocate(gbm, drm_fd, target.width(), target.height())
            .map_err(|error| {
                VulkanCpuScanoutError(format!("GBM framebuffer allocation failed: {error}"))
            })?;
        if let Err(error) = pool.write_rgba8(candidate, &pixels, target.width(), target.height()) {
            let _ = pool.retire(candidate);
            return Err(VulkanCpuScanoutError(format!(
                "GBM pixel upload failed: {error}"
            )));
        }

        let framebuffer = match pool.framebuffer(candidate) {
            Ok(framebuffer) => framebuffer.clone(),
            Err(error) => {
                let _ = pool.retire(candidate);
                return Err(VulkanCpuScanoutError(format!(
                    "GBM framebuffer lookup failed: {error}"
                )));
            }
        };

        let source: Size<i32, Buffer> = Size::from((width, height));
        let destination: Size<i32, Physical> = Size::from((width, height));
        let planes = [PlaneState {
            handle: surface.plane(),
            config: Some(PlaneConfig {
                src: Rectangle::from_size(source).to_f64(),
                dst: Rectangle::from_size(destination),
                transform: Transform::Normal,
                alpha: 1.0,
                damage_clips: None,
                fb: framebuffer,
                fence: None,
            }),
        }];

        if let Err(error) = presenter.submit_after_upload(surface, pool, candidate, &planes) {
            let _ = pool.retire(candidate);
            return Err(VulkanCpuScanoutError(format!(
                "CPU KMS presentation failed: {error}"
            )));
        }

        Ok(())
    }
}
