// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! First Vulkan-to-DRM scanout path using an owned DMA-BUF image.

use std::{error::Error, fmt};

use smithay::{
    backend::{
        allocator::Fourcc,
        drm::{DrmSurface, PlaneConfig, PlaneState},
    },
    utils::{Buffer, Physical, Rectangle, Size, Transform},
};

use crate::{
    drm::{
        DrmPrimaryPlaneFormatCapabilities, device::DrmDevice, gbm::GbmDeviceManager,
        presentation::DrmFramebufferId, presentation_backend::DrmPresentationBackend,
        surfaces::ManagedDrmSurface,
    },
    render::RenderScene,
    renderer::{RenderBackend, RenderTarget},
    vulkan::{
        DrmVulkanCompatibility, VulkanBackend, VulkanClearPass, VulkanCommandBuffer,
        VulkanDrmFramePool, VulkanDrmScanoutFrame, VulkanExternalMemoryCapabilities,
        VulkanFormatCapabilities, VulkanImageState, VulkanImageTransition, VulkanImageView,
        VulkanLogicalDevice,
    },
};

const FIRST_FRAME_CLEAR_COLOR: [f32; 4] = [0.035, 0.045, 0.065, 1.0];

/// Error returned by the initial Vulkan scanout path.
#[derive(Debug)]
pub(crate) struct VulkanScanoutError {
    message: String,
}

impl VulkanScanoutError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for VulkanScanoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for VulkanScanoutError {}

/// Presents a clear-only Vulkan frame through the output's native DRM surface.
///
/// The scene is intentionally not composited yet. This path proves the complete
/// Vulkan image, DMA-BUF, GBM framebuffer and KMS presentation lifecycle before
/// Wayland surface rendering is connected.
pub(crate) struct VulkanOutputScanoutBackend<'a> {
    drm_device: &'a DrmDevice,
    gbm_devices: &'a GbmDeviceManager,
    output: &'a mut ManagedDrmSurface,
    vulkan: &'a VulkanBackend,
}

impl<'a> VulkanOutputScanoutBackend<'a> {
    pub(crate) fn new(
        drm_device: &'a DrmDevice,
        gbm_devices: &'a GbmDeviceManager,
        output: &'a mut ManagedDrmSurface,
        vulkan: &'a VulkanBackend,
    ) -> Self {
        Self {
            drm_device,
            gbm_devices,
            output,
            vulkan,
        }
    }
}

impl RenderBackend for VulkanOutputScanoutBackend<'_> {
    type Error = VulkanScanoutError;

    fn render_frame(
        &mut self,
        target: &RenderTarget,
        _scene: &RenderScene<'_>,
    ) -> Result<(), Self::Error> {
        if target.output_id() != self.output.output_id() {
            return Err(VulkanScanoutError::new(
                "Render target does not belong to the managed DRM surface",
            ));
        }
        if target.width() == 0 || target.height() == 0 {
            return Err(VulkanScanoutError::new(
                "Cannot render a zero-sized DRM output",
            ));
        }
        if !self.drm_device.is_active() {
            return Err(VulkanScanoutError::new(
                "Cannot render while the DRM device is inactive",
            ));
        }
        if !self.vulkan.device().supports_foreign_queue_family() {
            return Err(VulkanScanoutError::new(
                "The selected Vulkan device does not support VK_EXT_queue_family_foreign",
            ));
        }

        let (surface, frame_pool, presentation) = self.output.scanout_parts_mut();
        if surface.is_legacy() {
            return Err(VulkanScanoutError::new(
                "The initial Vulkan scanout path requires atomic DRM/KMS",
            ));
        }

        let plane = DrmPrimaryPlaneFormatCapabilities {
            crtc_id: u32::from(surface.crtc()),
            plane_id: u32::from(surface.plane()),
            formats: surface.plane_info().formats.iter().copied().collect(),
        };

        let external_memory =
            VulkanExternalMemoryCapabilities::from_device(self.vulkan.selected_physical_device());
        if !external_memory.supports_dmabuf_export {
            return Err(VulkanScanoutError::new(format!(
                "Vulkan DMA-BUF export is unavailable; missing extensions: {}",
                external_memory.missing_extensions.join(", ")
            )));
        }

        let mut vulkan_formats = Vec::new();
        for drm_format in &plane.formats {
            if vulkan_formats
                .iter()
                .any(|capability: &VulkanFormatCapabilities| capability.format == drm_format.code)
            {
                continue;
            }

            let capabilities = VulkanFormatCapabilities::query(
                self.vulkan.selected_physical_device(),
                drm_format.code,
            )
            .map_err(VulkanScanoutError::new)?;
            vulkan_formats.push(capabilities);
        }

        let compatibility =
            DrmVulkanCompatibility::evaluate(&plane, &vulkan_formats, &external_memory);
        let selected_format = compatibility
            .formats()
            .iter()
            .find(|format| format.code == Fourcc::Xrgb8888)
            .or_else(|| {
                compatibility
                    .formats()
                    .iter()
                    .find(|format| format.code == Fourcc::Argb8888)
            })
            .or_else(|| compatibility.formats().first())
            .copied()
            .ok_or_else(|| {
                VulkanScanoutError::new(format!(
                    "No Vulkan color-attachment format is shared with the primary DRM plane: {:?}",
                    compatibility.status()
                ))
            })?;

        let frame = VulkanDrmScanoutFrame::allocate(
            self.vulkan.shared_device(),
            self.drm_device,
            self.gbm_devices,
            target.width(),
            target.height(),
            selected_format,
            smithay::reexports::ash::vk::ImageUsageFlags::COLOR_ATTACHMENT,
        )
        .map_err(|error| VulkanScanoutError::new(error.to_string()))?;

        let framebuffer = frame.framebuffer_id();
        frame_pool
            .insert(frame, presentation.frame_coordinator_mut())
            .map_err(|error| VulkanScanoutError::new(error.to_string()))?;

        if let Err(error) = render_and_present(
            surface,
            frame_pool,
            presentation,
            framebuffer,
            self.vulkan.device(),
            target.width(),
            target.height(),
        ) {
            let acquire_result = frame_pool
                .acquire_after_foreign_release(framebuffer, presentation.frame_coordinator_mut());

            let cleanup_result = match acquire_result {
                Ok(()) => frame_pool
                    .retire(framebuffer, presentation.frame_coordinator_mut())
                    .map(|_| ())
                    .map_err(|retire_error| retire_error.to_string()),
                Err(acquire_error) => match frame_pool
                    .retire(framebuffer, presentation.frame_coordinator_mut())
                {
                    Ok(_) => Ok(()),
                    Err(retire_error) => {
                        Err(format!("{acquire_error}; retirement also failed: {retire_error}"))
                    }
                },
            };

            if let Err(cleanup_error) = cleanup_result {
                eprintln!(
                    "AureaOS: retaining failed scanout framebuffer {}: {cleanup_error}",
                    framebuffer.value()
                );
            }

            return Err(error);
        }

        Ok(())
    }
}

fn render_and_present(
    surface: &DrmSurface,
    frame_pool: &VulkanDrmFramePool,
    presentation: &mut DrmPresentationBackend,
    framebuffer: DrmFramebufferId,
    device: &VulkanLogicalDevice,
    width: u32,
    height: u32,
) -> Result<(), VulkanScanoutError> {
    let width_i32 = i32::try_from(width)
        .map_err(|_| VulkanScanoutError::new("DRM output width exceeds supported coordinates"))?;
    let height_i32 = i32::try_from(height)
        .map_err(|_| VulkanScanoutError::new("DRM output height exceeds supported coordinates"))?;

    let (image, framebuffer_handle) = {
        let frame = frame_pool.get(framebuffer).ok_or_else(|| {
            VulkanScanoutError::new("The allocated scanout frame disappeared from its pool")
        })?;
        (frame.image(), frame.framebuffer_handle())
    };

    let view = VulkanImageView::color_drm(image)
        .map_err(|error| VulkanScanoutError::new(error.to_string()))?;
    let mut command_buffer = VulkanCommandBuffer::new(device).map_err(|error| {
        VulkanScanoutError::new(format!("Vulkan command allocation failed: {error:?}"))
    })?;

    command_buffer
        .begin(smithay::reexports::ash::vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
        .map_err(|error| {
            VulkanScanoutError::new(format!("Vulkan command begin failed: {error:?}"))
        })?;

    VulkanClearPass::record_color(
        command_buffer.handle(),
        &view,
        VulkanImageState::Undefined,
        VulkanImageState::General,
        FIRST_FRAME_CLEAR_COLOR,
    )
    .map_err(|error| VulkanScanoutError::new(error.to_string()))?;

    VulkanImageTransition::record_release_to_foreign(
        device,
        command_buffer.handle(),
        image.image(),
        VulkanImageState::General,
    )
    .map_err(|error| VulkanScanoutError::new(error.to_string()))?;

    command_buffer.end().map_err(|error| {
        VulkanScanoutError::new(format!("Vulkan command end failed: {error:?}"))
    })?;

    {
        let coordinator = presentation.frame_coordinator_mut();
        coordinator
            .begin_render(framebuffer)
            .map_err(|error| VulkanScanoutError::new(error.to_string()))?;
        coordinator
            .mark_submitted(framebuffer)
            .map_err(|error| VulkanScanoutError::new(error.to_string()))?;
    }

    command_buffer.submit_and_wait().map_err(|error| {
        VulkanScanoutError::new(format!("Vulkan queue submission failed: {error:?}"))
    })?;

    {
        let coordinator = presentation.frame_coordinator_mut();
        coordinator
            .mark_gpu_complete(framebuffer)
            .map_err(|error| VulkanScanoutError::new(error.to_string()))?;
        coordinator
            .mark_foreign_release_submitted(framebuffer)
            .map_err(|error| VulkanScanoutError::new(error.to_string()))?;
    }

    drop(view);

    let source_size: Size<i32, Buffer> = Size::from((width_i32, height_i32));
    let destination_size: Size<i32, Physical> = Size::from((width_i32, height_i32));
    let plane_state = PlaneState {
        handle: surface.plane(),
        config: Some(PlaneConfig {
            src: Rectangle::from_size(source_size).to_f64(),
            dst: Rectangle::from_size(destination_size),
            transform: Transform::Normal,
            alpha: 1.0,
            damage_clips: None,
            fb: framebuffer_handle,
            fence: None,
        }),
    };

    presentation
        .submit_after_gpu_completion(surface, framebuffer, &[plane_state])
        .map_err(|error| VulkanScanoutError::new(error.to_string()))
}
