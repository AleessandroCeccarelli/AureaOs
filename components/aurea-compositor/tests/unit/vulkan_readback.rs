// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use aurea_compositor::vulkan::{VulkanOffscreenError, VulkanOffscreenRenderer};

#[test]
fn rejects_zero_width() {
    assert!(matches!(
        VulkanOffscreenRenderer::validate_dimensions(0, 64),
        Err(VulkanOffscreenError::InvalidDimensions),
    ));
}

#[test]
fn rejects_zero_height() {
    assert!(matches!(
        VulkanOffscreenRenderer::validate_dimensions(64, 0),
        Err(VulkanOffscreenError::InvalidDimensions),
    ));
}

#[test]
fn accepts_non_zero_dimensions() {
    assert!(VulkanOffscreenRenderer::validate_dimensions(64, 64).is_ok());
}
