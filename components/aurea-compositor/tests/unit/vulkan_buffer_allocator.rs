// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Hardware-independent tests for Vulkan render-target request validation.

use aurea_compositor::vulkan::{DrmVulkanFormat, VulkanBufferError, VulkanRenderTargetRequest};
use smithay::backend::allocator::{Fourcc, Modifier};

fn format() -> DrmVulkanFormat {
    DrmVulkanFormat {
        code: Fourcc::Argb8888,
        modifier: Modifier::Linear,
        memory_plane_count: 1,
    }
}

#[test]
fn rejects_zero_width_render_target() {
    let result = VulkanRenderTargetRequest::new(0, 1080, format());

    assert!(matches!(
        result,
        Err(VulkanBufferError::InvalidDimensions {
            width: 0,
            height: 1080
        })
    ));
}

#[test]
fn rejects_zero_height_render_target() {
    let result = VulkanRenderTargetRequest::new(1920, 0, format());

    assert!(matches!(
        result,
        Err(VulkanBufferError::InvalidDimensions {
            width: 1920,
            height: 0
        })
    ));
}

#[test]
fn accepts_non_zero_render_target_dimensions() {
    let request = VulkanRenderTargetRequest::new(1920, 1080, format())
        .expect("non-zero dimensions should be accepted");

    assert_eq!(request.width(), 1920);
    assert_eq!(request.height(), 1080);
    assert_eq!(request.format(), format());
}
