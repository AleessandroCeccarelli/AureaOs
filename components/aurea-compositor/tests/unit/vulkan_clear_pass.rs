// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use aurea_compositor::vulkan::{VulkanClearPass, VulkanClearPassError, VulkanImageState};

#[test]
fn accepts_finite_clear_color_and_defined_final_state() {
    assert_eq!(
        VulkanClearPass::validate([0.08, 0.12, 0.18, 1.0], VulkanImageState::TransferSource,),
        Ok(()),
    );
}

#[test]
fn rejects_non_finite_clear_color() {
    assert_eq!(
        VulkanClearPass::validate([0.0, f32::NAN, 0.0, 1.0], VulkanImageState::TransferSource,),
        Err(VulkanClearPassError::NonFiniteClearColor),
    );
}

#[test]
fn rejects_undefined_final_state() {
    assert_eq!(
        VulkanClearPass::validate([0.0, 0.0, 0.0, 1.0], VulkanImageState::Undefined,),
        Err(VulkanClearPassError::UndefinedFinalState),
    );
}
