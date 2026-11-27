// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan frame composition, ownership release and atomic KMS submission.

use smithay::{
    backend::drm::{DrmSurface, PlaneConfig, PlaneState},
    utils::{Buffer, Physical, Rectangle, Size, Transform},
};

use crate::{
    drm::{presentation::DrmFramebufferId, presentation_backend::DrmPresentationBackend},
    render::RenderScene,
    renderer::RenderTarget,
    vulkan::{
        VulkanDisplayColorManager, VulkanDrmFramePool, VulkanImageState, VulkanImageView,
        VulkanLogicalDevice, VulkanSurfacePipelineCache, surface_compositor,
    },
};

use super::VulkanScanoutError;

pub(super) fn render_and_present(
    surface: &DrmSurface,
    frame_pool: &mut VulkanDrmFramePool,
    presentation: &mut DrmPresentationBackend,
    framebuffer: DrmFramebufferId,
    device: &VulkanLogicalDevice,
    target: &RenderTarget,
    scene: &RenderScene<'_>,
    previous_image_state: VulkanImageState,
    display_color_manager: &mut VulkanDisplayColorManager,
    surface_cache: &mut VulkanSurfacePipelineCache,
    warmth_percent: u8,
    shared_device: std::sync::Arc<VulkanLogicalDevice>,
) -> Result<(), VulkanScanoutError> {
    let width_i32 = i32::try_from(target.width())
        .map_err(|_| VulkanScanoutError::new("DRM output width exceeds supported coordinates"))?;
    let height_i32 = i32::try_from(target.height())
        .map_err(|_| VulkanScanoutError::new("DRM output height exceeds supported coordinates"))?;

    let display_pass = if warmth_percent > 0 {
        let format = frame_pool
            .get(framebuffer)
            .ok_or_else(|| {
                VulkanScanoutError::new("The allocated scanout frame disappeared from its pool")
            })?
            .image()
            .format();
        let pass = display_color_manager.pass_for_format(format).ok();
        if pass.is_some() {
            if let Some(frame) = frame_pool.get_mut(framebuffer) {
                frame.ensure_intermediate(std::sync::Arc::clone(&shared_device));
            }
        }
        pass
    } else {
        None
    };

    let (image, framebuffer_handle, intermediate) = {
        let frame = frame_pool.get(framebuffer).ok_or_else(|| {
            VulkanScanoutError::new("The allocated scanout frame disappeared from its pool")
        })?;
        (
            frame.image(),
            frame.framebuffer_handle(),
            frame.intermediate_image().zip(frame.intermediate_state()),
        )
    };

    let view = VulkanImageView::color_drm(image)
        .map_err(|error| VulkanScanoutError::new(error.to_string()))?;

    let intermediate_view = intermediate
        .as_ref()
        .and_then(|(image, _)| VulkanImageView::color_shared(image).ok());
    let effective_display = display_pass
        .zip(intermediate_view.as_ref())
        .zip(intermediate.as_ref().map(|(_, state)| *state))
        .map(|((pass, intermediate_view), state)| (pass, warmth_percent, intermediate_view, state));

    presentation
        .frame_coordinator_mut()
        .begin_render(framebuffer)
        .map_err(|error| VulkanScanoutError::new(error.to_string()))?;

    if let Err(error) = surface_compositor::render_and_release(
        device,
        &view,
        target,
        scene,
        surface_cache,
        previous_image_state,
        effective_display,
        || {
            presentation
                .frame_coordinator_mut()
                .mark_submitted(framebuffer)
                .expect("frame must remain Rendering until submission acceptance is resolved");
        },
    ) {
        match error {
            crate::vulkan::surface_compositor::VulkanSurfaceCompositionError::UncertainSubmission(error) => {
                // The in-flight command buffer may still reference these views.
                // The images remain owned by the DRM frame pool; quarantine the
                // transient views until process/device teardown instead of destroying them.
                std::mem::forget(view);
                if let Some(intermediate_view) = intermediate_view {
                    std::mem::forget(intermediate_view);
                }
                return Err(VulkanScanoutError::device_lost(format!(
                    "Vulkan submission completion is uncertain: {error:?}"
                )));
            }
            crate::vulkan::surface_compositor::VulkanSurfaceCompositionError::DeviceLost(result) => {
                return Err(VulkanScanoutError::device_lost(format!(
                    "Vulkan device lost: {result:?}"
                )));
            }
            other => return Err(VulkanScanoutError::new(other.to_string())),
        }
    }

    if effective_display.is_some() {
        if let Some(frame) = frame_pool.get(framebuffer) {
            frame.set_intermediate_state(VulkanImageState::ShaderRead);
        }
    }

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
