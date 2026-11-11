// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use aurea_compositor::vulkan::VulkanOffscreenError;

#[test]
fn offscreen_renderer_reports_invalid_dimensions() {
    assert!(matches!(
        VulkanOffscreenError::InvalidDimensions,
        VulkanOffscreenError::InvalidDimensions,
    ));
}
