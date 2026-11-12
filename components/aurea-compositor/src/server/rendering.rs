// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Dispatches requested output frames to the DRM/Vulkan scanout backend.

use crate::{drm::scanout_renderer::VulkanOutputScanoutBackend, server::state::WaylandState};

pub(super) fn render_pending_outputs(state: &mut WaylandState) {
    let WaylandState {
        compositor,
        drm_devices,
        drm_surfaces,
        gbm_devices,
        vulkan_backends,
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
            compositor.cancel_output_frame(output_id);
            continue;
        };
        if !gbm_devices.contains(node) {
            compositor.cancel_output_frame(output_id);
            continue;
        }
        let Some(vulkan) = vulkan_backends.get(&node) else {
            compositor.cancel_output_frame(output_id);
            continue;
        };
        let Some(surface_manager) = drm_surfaces.get_mut(&node) else {
            compositor.cancel_output_frame(output_id);
            continue;
        };
        let Some(output) = surface_manager.get_mut(output_id) else {
            compositor.cancel_output_frame(output_id);
            continue;
        };
        if output.presentation().pending().is_some() {
            continue;
        }

        let mut backend = VulkanOutputScanoutBackend::new(device, gbm_devices, output, vulkan);
        if let Err(error) = compositor.render_output(&mut backend, output_id) {
            eprintln!("AureaOS: output {output_id} rendering failed: {error}");
        }
    }
}
