// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Dispatches requested output frames to the DRM/Vulkan scanout backend.

use std::io;

use crate::{drm::scanout_renderer::VulkanOutputScanoutBackend, server::state::WaylandState};

pub(super) fn render_pending_outputs(state: &mut WaylandState) -> io::Result<()> {
    let WaylandState {
        compositor,
        drm_devices,
        drm_surfaces,
        gbm_devices,
        vulkan_backends,
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

    for (node, output_id) in pending_outputs {
        let Some(device) = drm_devices.get(node) else {
            return Err(io::Error::other(format!(
                "AureaOS: DRM device {node} disappeared while output {output_id} had a pending frame"
            )));
        };
        if !gbm_devices.contains(node) {
            return Err(io::Error::other(format!(
                "AureaOS: GBM is unavailable for DRM device {node} while output {output_id} had a pending frame"
            )));
        }
        let Some(vulkan) = vulkan_backends.get(&node) else {
            return Err(io::Error::other(format!(
                "AureaOS: Vulkan backend is unavailable for DRM device {node} while output {output_id} had a pending frame"
            )));
        };
        let Some(surface_manager) = drm_surfaces.get_mut(&node) else {
            return Err(io::Error::other(format!(
                "AureaOS: DRM surface manager disappeared for output {output_id}"
            )));
        };
        let Some(output) = surface_manager.get_mut(output_id) else {
            return Err(io::Error::other(format!(
                "AureaOS: DRM surface disappeared for output {output_id}"
            )));
        };
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
        compositor
            .render_output(&mut backend, output_id)
            .map_err(|error| {
                io::Error::other(format!(
                    "AureaOS: output {output_id} rendering or presentation failed: {error}"
                ))
            })?;
    }

    Ok(())
}
