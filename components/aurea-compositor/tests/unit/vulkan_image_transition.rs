// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use aurea_compositor::vulkan::VulkanImageState;

#[test]
fn renderer_image_states_are_explicit() {
    let states = [
        VulkanImageState::Undefined,
        VulkanImageState::ColorAttachment,
        VulkanImageState::ShaderRead,
        VulkanImageState::TransferSource,
        VulkanImageState::TransferDestination,
        VulkanImageState::General,
    ];

    for (index, state) in states.iter().enumerate() {
        assert!(!states[..index].contains(state));
    }
}
