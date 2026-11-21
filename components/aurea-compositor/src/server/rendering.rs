// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Dispatches requested output frames to the DRM/Vulkan scanout backend.

use std::{
    io,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static RENDERED_OUTPUT_FRAMES: AtomicU64 = AtomicU64::new(0);

use crate::{
    drm::{
        cpu_scanout_renderer::VulkanCpuOutputBackend, scanout_renderer::VulkanOutputScanoutBackend,
    },
    presentation_selector::{
        NoCompatiblePresentationPath, PresentationBackendSupport, PresentationPath,
        PresentationSelector,
    },
    server::state::WaylandState,
};

pub(super) fn render_pending_outputs(state: &mut WaylandState) -> io::Result<()> {
    let WaylandState {
        compositor,
        drm_devices,
        drm_surfaces,
        gbm_devices,
        vulkan_backends,
        gpu_manager,
        drm_device_registry,
        display_color,
        ..
    } = state;

    let mut pending_outputs = Vec::new();
    for (node, surfaces) in drm_surfaces.iter() {
        for (output_id, _) in surfaces.iter() {
            if compositor.has_pending_output_frame(*output_id) {
                pending_outputs.push((*node, *output_id));
            }
        }
    }

    for (presentation_node, output_id) in pending_outputs {
        let Some(device) = drm_devices.get(presentation_node) else {
            return Err(io::Error::other(format!(
                "AureaOS: DRM device {presentation_node} disappeared while output {output_id} had a pending frame"
            )));
        };
        let render_path: PathBuf = gpu_manager
            .selected_render_device_for_output_if(device.device_path(), |candidate| {
                drm_device_registry
                    .node_for_path(candidate.path())
                    .map(|node| vulkan_backends.contains_key(&node))
                    .unwrap_or(false)
            })
            .map(|selected| selected.path().to_path_buf())
            .ok_or_else(|| {
                io::Error::other(format!(
                    "AureaOS: no available GPU has an initialized Vulkan renderer for output {output_id}"
                ))
            })?;
        let rendering_node = drm_device_registry
            .node_for_path(&render_path)
            .ok_or_else(|| {
                io::Error::other(format!(
                    "AureaOS: selected rendering GPU {} has no registered DRM identity",
                    render_path.display()
                ))
            })?;

        // Both current presentation backends require atomic KMS. Check this
        // before routing so legacy surfaces are rejected during capability
        // selection, rather than failing after a frame has been requested.
        let supports_atomic_kms = drm_surfaces
            .get(&presentation_node)
            .and_then(|manager| manager.get(output_id))
            .map(|output| !output.surface().is_legacy())
            .unwrap_or(false);

        // Direct Vulkan scanout additionally requires the same DRM device and
        // the full DMA-BUF/foreign-ownership capability set. The compatibility
        // route composes on the render GPU and uploads host-readback pixels via GBM/KMS.
        let support = [PresentationBackendSupport {
            rendering_gpu: rendering_node,
            presentation_gpu: presentation_node,
            direct_vulkan: supports_atomic_kms
                && drm_surfaces
                    .get(&presentation_node)
                    .and_then(|manager| manager.get(output_id))
                    .map(|output| !output.direct_scanout_disabled())
                    .unwrap_or(false)
                && rendering_node == presentation_node
                && vulkan_backends
                    .get(&rendering_node)
                    .map(|backend| backend.supports_direct_presentation())
                    .unwrap_or(false),
            // The compatibility backend requires atomic KMS, GBM on the
            // presentation GPU, and Vulkan on the selected rendering GPU.
            compatibility: supports_atomic_kms
                && gbm_devices.contains(presentation_node)
                && vulkan_backends.contains_key(&rendering_node),
        }];
        let selected_path = PresentationSelector::select(
            &rendering_node,
            &presentation_node,
            &support,
        )
        .map_err(|NoCompatiblePresentationPath| {
            io::Error::other(format!(
                "AureaOS: no presentation backend supports rendering GPU {rendering_node} and output GPU {presentation_node} for output {output_id}"
            ))
        })?;

        if !gbm_devices.contains(presentation_node) {
            return Err(io::Error::other(format!(
                "AureaOS: GBM is unavailable for DRM device {presentation_node} while output {output_id} had a pending frame"
            )));
        }
        let Some(vulkan) = vulkan_backends.get(&rendering_node) else {
            return Err(io::Error::other(format!(
                "AureaOS: Vulkan rendering backend is unavailable for {rendering_node}"
            )));
        };
        let Some(surface_manager) = drm_surfaces.get_mut(&presentation_node) else {
            return Err(io::Error::other(format!(
                "AureaOS: DRM surface manager disappeared for output {output_id}"
            )));
        };
        let Some(output) = surface_manager.get_mut(output_id) else {
            return Err(io::Error::other(format!(
                "AureaOS: DRM surface disappeared for output {output_id}"
            )));
        };

        let path_name = if matches!(&selected_path, PresentationPath::DirectVulkan) {
            "direct-vulkan"
        } else {
            "cpu-compatibility"
        };
        eprintln!(
            "AureaOS render: pending output={} render_gpu={} presentation_gpu={} path={}",
            output_id, rendering_node, presentation_node, path_name
        );

        match selected_path {
            PresentationPath::DirectVulkan => {
                if output.presentation().pending().is_some() {
                    continue;
                }
                let warmth_percent = display_color.adjustment().warmth_percent();
                let mut display_color_manager = vulkan
                    .display_color_manager()
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let mut surface_pipeline_cache = vulkan
                    .surface_pipeline_cache()
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let mut backend = VulkanOutputScanoutBackend::new(
                    device,
                    gbm_devices,
                    output,
                    vulkan,
                    &mut display_color_manager,
                    &mut surface_pipeline_cache,
                    warmth_percent,
                );
                let direct_result = compositor.render_output(&mut backend, output_id);
                drop(backend);

                if let Err(error) = direct_result {
                    if output.presentation().pending().is_some() {
                        return Err(io::Error::other(format!(
                            "AureaOS: direct Vulkan presentation for output {output_id} failed after KMS submission; refusing an unsafe CPU retry: {error}"
                        )));
                    }
                    output.disable_direct_scanout();
                    // Extension support alone does not guarantee that this
                    // connector accepts a Vulkan scanout format/modifier.
                    // Retry through host readback + GBM upload instead of
                    // terminating the entire Wayland session.
                    eprintln!(
                        "AureaOS: direct Vulkan presentation failed for output {output_id}: {error}; retrying with CPU-upload compatibility"
                    );
                    compositor
                        .request_output_frame(output_id)
                        .map_err(io::Error::other)?;

                    if output.cpu_presentation_pending() {
                        continue;
                    }

                    let mut compatibility_backend = VulkanCpuOutputBackend::new(
                        device,
                        gbm_devices,
                        output,
                        vulkan,
                        &mut surface_pipeline_cache,
                        &mut display_color_manager,
                        warmth_percent,
                    );
                    compositor
                        .render_output(&mut compatibility_backend, output_id)
                        .map_err(|error| {
                            io::Error::other(format!(
                                "AureaOS: direct Vulkan and CPU compatibility presentation both failed for output {output_id}: {error}"
                            ))
                        })?;
                }
            }
            PresentationPath::Compatibility => {
                // Keep the compositor's frame pending until the CPU-upload
                // framebuffer is released by its KMS page-flip event.
                if output.cpu_presentation_pending() {
                    continue;
                }
                let warmth_percent = display_color.adjustment().warmth_percent();
                let mut display_color_manager = vulkan
                    .display_color_manager()
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let mut surface_pipeline_cache = vulkan
                    .surface_pipeline_cache()
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let mut backend = VulkanCpuOutputBackend::new(
                    device,
                    gbm_devices,
                    output,
                    vulkan,
                    &mut surface_pipeline_cache,
                    &mut display_color_manager,
                    warmth_percent,
                );
                compositor
                    .render_output(&mut backend, output_id)
                    .map_err(|error| {
                        io::Error::other(format!(
                            "AureaOS: CPU compatibility output {output_id} failed: {error}"
                        ))
                    })?;
            }
        }

        let frame_count = RENDERED_OUTPUT_FRAMES.fetch_add(1, Ordering::Relaxed) + 1;
        if frame_count <= 4 || frame_count % 120 == 0 {
            eprintln!(
                "AureaOS render: render_output completed count={} output={} path={}",
                frame_count, output_id, path_name
            );
        }
    }

    Ok(())
}
