// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use aurea_compositor::vulkan::{VulkanShaderError, VulkanShaderModule};

#[test]
fn spirv_validation_rejects_empty_modules() {
    assert_eq!(
        VulkanShaderModule::validate_words(&[]),
        Err(VulkanShaderError::EmptyCode)
    );
}

#[test]
fn spirv_validation_rejects_invalid_magic() {
    assert_eq!(
        VulkanShaderModule::validate_words(&[0, 1, 2, 3]),
        Err(VulkanShaderError::InvalidMagic)
    );
}

#[test]
fn spirv_validation_accepts_valid_magic_prefix() {
    assert_eq!(
        VulkanShaderModule::validate_words(&[0x0723_0203, 0x0001_0000]),
        Ok(())
    );
}
