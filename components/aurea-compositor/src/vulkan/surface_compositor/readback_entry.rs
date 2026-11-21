// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Readback entry point for the shared Wayland scene compositor.

use super::super::{
    VulkanDisplayColorPass, VulkanImageState, VulkanImageView, VulkanLogicalDevice,
    VulkanReadbackBuffer, VulkanSurfacePipelineCache,
};
use crate::{render::RenderScene, renderer::RenderTarget};

use super::{render_frame, VulkanSurfaceCompositionError};

/// Composes the same Wayland scene as the direct path, then copies RGBA8 pixels
/// to host-visible memory instead of transferring image ownership to KMS.
pub(crate) fn render_and_readback(
    device: &VulkanLogicalDevice,
    view: &VulkanImageView<'_, '_>,
    target: &RenderTarget,
    scene: &RenderScene<'_>,
    surface_cache: &mut VulkanSurfacePipelineCache,
    previous_image_state: VulkanImageState,
    display_color: Option<(
        &VulkanDisplayColorPass<'_>,
        u8,
        &VulkanImageView<'_, '_>,
        VulkanImageState,
    )>,
    readback: &VulkanReadbackBuffer<'_>,
) -> Result<(), VulkanSurfaceCompositionError> {
    render_frame(
        device, view, target, scene, surface_cache, previous_image_state,
        display_color, Some(readback), || {},
    )
}
