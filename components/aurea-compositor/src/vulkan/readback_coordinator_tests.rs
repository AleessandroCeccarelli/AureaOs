// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Unit tests for readback coordinator configuration.

use super::VulkanReadbackCoordinator;
use crate::vulkan::VulkanReadbackError;

#[test]
fn coordinator_rejects_zero_in_flight_limit() {
    assert_eq!(
        VulkanReadbackCoordinator::validate_limit(0),
        Err(VulkanReadbackError::InvalidCoordinatorLimit)
    );
}

#[test]
fn coordinator_accepts_positive_in_flight_limit() {
    assert_eq!(VulkanReadbackCoordinator::validate_limit(1), Ok(()));
    assert_eq!(VulkanReadbackCoordinator::validate_limit(3), Ok(()));
}
