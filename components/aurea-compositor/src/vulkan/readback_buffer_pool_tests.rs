// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Unit tests for readback-pool size validation.

use super::readback_byte_len;
use crate::vulkan::VulkanReadbackError;

mod tests {
    use super::readback_byte_len;
    use crate::vulkan::VulkanReadbackError;

    #[test]
    fn readback_size_rejects_zero_dimensions() {
        assert_eq!(
            readback_byte_len(0, 1),
            Err(VulkanReadbackError::InvalidDimensions)
        );
        assert_eq!(
            readback_byte_len(1, 0),
            Err(VulkanReadbackError::InvalidDimensions)
        );
    }

    #[test]
    fn readback_size_uses_four_bytes_per_pixel() {
        assert_eq!(readback_byte_len(16, 8), Ok(512));
    }

    #[test]
    fn readback_size_rejects_overflow() {
        assert_eq!(
            readback_byte_len(u32::MAX, u32::MAX),
            Err(VulkanReadbackError::SizeOverflow)
        );
    }
}

