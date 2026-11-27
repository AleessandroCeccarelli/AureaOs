// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan renderer lifecycle for DRM-owned outputs.

use smithay::backend::drm::DrmNode;

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

    // linux-dmabuf v3 has one global format list. Advertise only the
    // intersection that every initialized renderer can import and sample.
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
    let display_handle = state.display.handle();
    if state.compositor.initialize_dmabuf_global(&display_handle, devices, formats) {
        tracing::info!("AureaOS: linux-dmabuf v3 global published with cross-GPU importable formats");
    } else {
        tracing::info!("AureaOS: linux-dmabuf disabled; no common safe DMA-BUF import formats");
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
