// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan renderer lifecycle for DRM-owned outputs.

use smithay::{
    backend::drm::DrmNode,
    wayland::dmabuf::DmabufFeedbackBuilder,
};

use crate::{
    gpu::GpuSelectionMetadata,
    renderer::RenderPreference,
    server::state::WaylandState,
    vulkan::{VulkanBackend, VulkanDeviceType, VulkanDmabufImportSupport},
};

/// Initializes one renderer for every opened DRM device.
pub(super) fn initialize_vulkan_backends(state: &mut WaylandState) {
    let nodes = state
        .drm_devices
        .iter()
        .map(|(node, _)| *node)
        .collect::<Vec<_>>();

    for node in nodes {
        initialize_vulkan_backend(state, node);
    }

    refresh_dmabuf_feedback(state);
}

/// Reconciles the feedback global and import validation set with active Vulkan devices.
pub(super) fn refresh_dmabuf_feedback(state: &mut WaylandState) {
    // The main tranche is intentionally conservative: every advertised pair
    // must be importable and sampleable on every active renderer.
    let devices = state
        .vulkan_backends
        .values()
        .map(VulkanBackend::shared_device)
        .collect::<Vec<_>>();
    let mut formats = devices
        .first()
        .map(|device| VulkanDmabufImportSupport::supported_formats(device))
        .unwrap_or_default();
    for device in devices.iter().skip(1) {
        let supported = VulkanDmabufImportSupport::supported_formats(device);
        formats.retain(|format| supported.contains(format));
    }
    // The protocol main_device must identify the compositor's selected
    // primary GPU, not an arbitrary HashMap entry (whose iteration order is
    // intentionally unspecified).
    let main_device = state
        .gpu_manager
        .primary_device()
        .and_then(|primary| {
            state
                .udev_device_ids
                .iter()
                .find(|(_, path)| path == primary.path())
                .map(|(node, _)| *node)
        })
        .and_then(|node| state.vulkan_backends.get(&node))
        .and_then(|backend| {
            let physical = backend.selected_physical_device();
            physical
                .primary_node()
                .ok()
                .flatten()
                .or_else(|| physical.render_node().ok().flatten())
                .map(|node| node.dev_id())
        });
    // An empty feedback table has no main-device tranche and is not a valid
    // replacement for this feedback global. Keep the last published table,
    // but fail closed by clearing import targets until a renderer returns.
    if devices.is_empty() || formats.is_empty() {
        state.compositor.clear_dmabuf_devices();
        tracing::info!("AureaOS: DMA-BUF imports disabled; no active renderer intersection is available");
        return;
    }
    let main_device = main_device.or_else(|| state.compositor.dmabuf_main_device());
    let Some(main_device) = main_device else {
        state.compositor.clear_dmabuf_devices();
        tracing::warn!("AureaOS: linux-dmabuf feedback not refreshed; no known DRM device identity");
        return;
    };
    let feedback = match DmabufFeedbackBuilder::new(main_device, formats).build() {
        Ok(feedback) => feedback,
        Err(error) => {
            tracing::warn!(%error, "AureaOS: unable to build linux-dmabuf default feedback");
            return;
        }
    };
    let display_handle = state.display.handle();
    if state.compositor.initialize_dmabuf_global(&display_handle, devices, feedback, main_device) {
        tracing::info!("AureaOS: linux-dmabuf feedback reconciled with active Vulkan renderers");
    }
}

/// Initializes a DRM-matched renderer, falling back to a Vulkan CPU device.
pub(super) fn initialize_vulkan_backend(state: &mut WaylandState, node: DrmNode) {
    if state.vulkan_backends.contains_key(&node) {
        return;
    }

    let backend = match VulkanBackend::new_for_drm_node(node, RenderPreference::Automatic) {
        Ok(backend) => backend,
        Err(hardware_error) => {
            eprintln!(
                "AureaOS: DRM-matched Vulkan renderer unavailable for {node}: {hardware_error}"
            );
            match VulkanBackend::new_software_fallback() {
                Ok(backend) => {
                    eprintln!(
                        "AureaOS: using Vulkan CPU rendering with GBM/KMS presentation on {node}"
                    );
                    backend
                }
                Err(software_error) => {
                    eprintln!(
                        "AureaOS: Vulkan software fallback unavailable for {node}: {software_error}"
                    );
                    return;
                }
            }
        }
    };

    update_hardware_rendering_metadata(state, node, &backend);
    state.vulkan_backends.insert(node, backend);
    state.gpu_manager.select_primary_by_policy();
    refresh_dmabuf_feedback(state);
}

fn update_hardware_rendering_metadata(
    state: &mut WaylandState,
    node: DrmNode,
    backend: &VulkanBackend,
) {
    let hardware_rendering = !matches!(
        backend.selected_device_info().device_type,
        VulkanDeviceType::Cpu
    );

    let Some((_, path)) = state
        .udev_device_ids
        .iter()
        .find(|(known, _)| *known == node)
    else {
        return;
    };
    let path = path.clone();

    let Some(device) = state.gpu_manager.get_device(&path) else {
        return;
    };
    let mut metadata: GpuSelectionMetadata = device.selection_metadata();
    metadata.hardware_rendering = hardware_rendering;

    if let Err(error) = state.gpu_manager.set_selection_metadata(&path, metadata) {
        eprintln!(
            "AureaOS: unable to update Vulkan GPU capability for {}: {error}",
            path.display()
        );
    }
}
