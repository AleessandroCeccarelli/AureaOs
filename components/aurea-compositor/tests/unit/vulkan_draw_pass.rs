// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use aurea_compositor::vulkan::{VulkanDrawPass, VulkanDrawPassError, VulkanImageState};

#[test]
fn accepts_a_non_empty_draw_with_defined_final_state() {
    assert_eq!(
        VulkanDrawPass::validate(3, 1, VulkanImageState::TransferSource),
        Ok(()),
    );
}

#[test]
fn rejects_zero_vertices() {
    assert_eq!(
        VulkanDrawPass::validate(0, 1, VulkanImageState::TransferSource),
        Err(VulkanDrawPassError::EmptyDraw),
    );
}

#[test]
fn rejects_zero_instances() {
    assert_eq!(
        VulkanDrawPass::validate(3, 0, VulkanImageState::TransferSource),
        Err(VulkanDrawPassError::EmptyDraw),
    );
}

#[test]
fn rejects_undefined_final_state() {
    assert_eq!(
        VulkanDrawPass::validate(3, 1, VulkanImageState::Undefined),
        Err(VulkanDrawPassError::UndefinedFinalState),
    );
}
