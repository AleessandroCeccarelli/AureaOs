// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Dispatches requested output frames to the DRM/Vulkan scanout backend.

use std::{io, path::PathBuf};

use crate::{
    drm::{
        cpu_scanout_renderer::VulkanCpuOutputBackend,
        scanout_renderer::VulkanOutputScanoutBackend,
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
            .selected_render_device_for_output(device.device_path())
            .map(|selected| selected.path().to_path_buf())
            .ok_or_else(|| {
                io::Error::other(format!(
                    "AureaOS: no available rendering GPU can be selected for output {output_id}"
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

        // Direct Vulkan scanout is currently implemented only when rendering
        // and presentation use the same DRM device. Cross-GPU compatibility
        // remains unavailable until its CPU-to-GBM backend is implemented.
        let support = [PresentationBackendSupport {
            rendering_gpu: rendering_node,
            presentation_gpu: presentation_node,
            direct_vulkan: rendering_node == presentation_node
                && vulkan_backends
                    .get(&rendering_node)
                    .map(|backend| backend.supports_direct_presentation())
                    .unwrap_or(false),
            compatibility: gbm_devices.contains(presentation_node)
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
                compositor.render_output(&mut backend, output_id).map_err(|error| {
                    io::Error::other(format!(
                        "AureaOS: direct Vulkan output {output_id} failed: {error}"
                    ))
                })?;
            }
            PresentationPath::Compatibility => {
                let mut backend =
                    VulkanCpuOutputBackend::new(device, gbm_devices, output, vulkan);
                compositor.render_output(&mut backend, output_id).map_err(|error| {
                    io::Error::other(format!(
                        "AureaOS: CPU compatibility output {output_id} failed: {error}"
                    ))
                })?;
            }
        }
    }

    Ok(())
}
