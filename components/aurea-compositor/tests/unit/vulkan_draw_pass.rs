// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use aurea_compositor::vulkan::{VulkanColorLoad, VulkanDrawPass, VulkanDrawPassError, VulkanImageState};

#[test]
fn accepts_a_non_empty_draw_with_defined_final_state() {
    assert_eq!(
        VulkanDrawPass::validate(3, 1, VulkanImageState::TransferSource, VulkanColorLoad::Clear([0.0, 0.0, 0.0, 1.0])),
        Ok(()),
    );
}

#[test]
fn rejects_zero_vertices() {
    assert_eq!(
        VulkanDrawPass::validate(0, 1, VulkanImageState::TransferSource, VulkanColorLoad::DontCare),
        Err(VulkanDrawPassError::EmptyDraw),
    );
}

#[test]
fn rejects_zero_instances() {
    assert_eq!(
        VulkanDrawPass::validate(3, 0, VulkanImageState::TransferSource, VulkanColorLoad::DontCare),
        Err(VulkanDrawPassError::EmptyDraw),
    );
}

#[test]
fn rejects_undefined_final_state() {
    assert_eq!(
        VulkanDrawPass::validate(3, 1, VulkanImageState::Undefined, VulkanColorLoad::DontCare),
        Err(VulkanDrawPassError::UndefinedFinalState),
    );
}

#[test]
fn rejects_non_finite_clear_color() {
    assert_eq!(
        VulkanDrawPass::validate(
            3,
            1,
            VulkanImageState::TransferSource,
            VulkanColorLoad::Clear([0.0, f32::NAN, 0.0, 1.0]),
        ),
        Err(VulkanDrawPassError::NonFiniteClearColor),
    );
}
