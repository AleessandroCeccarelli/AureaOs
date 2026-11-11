// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Validated SPIR-V shader-module ownership.

use std::{error::Error, fmt};

use smithay::reexports::ash::vk;

use super::VulkanLogicalDevice;

/// Errors returned while creating a shader module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanShaderError {
    EmptyCode,
    InvalidByteLength,
    InvalidMagic,
    InvalidHeader,
    Creation(vk::Result),
}

impl fmt::Display for VulkanShaderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCode => formatter.write_str("SPIR-V shader code is empty"),
            Self::InvalidByteLength => formatter.write_str("SPIR-V byte length is not divisible by four"),
            Self::InvalidMagic => formatter.write_str("SPIR-V module has an invalid magic number"),
            Self::InvalidHeader => formatter.write_str("SPIR-V module is shorter than its five-word header"),
            Self::Creation(error) => write!(formatter, "Vulkan shader-module creation failed: {error:?}"),
        }
    }
}

impl Error for VulkanShaderError {}

/// Owns one VkShaderModule created from validated SPIR-V words.
pub struct VulkanShaderModule<'device> {
    device: &'device VulkanLogicalDevice,
    handle: vk::ShaderModule,
}

impl<'device> VulkanShaderModule<'device> {
    /// Creates a shader module from SPIR-V words.
    pub fn from_words(
        device: &'device VulkanLogicalDevice,
        words: &[u32],
    ) -> Result<Self, VulkanShaderError> {
        Self::validate_words(words)?;

        let create_info = vk::ShaderModuleCreateInfo::default().code(words);
        let handle = unsafe {
            device.handle().create_shader_module(&create_info, None)
        }.map_err(VulkanShaderError::Creation)?;

        Ok(Self { device, handle })
    }

    /// Creates a shader module from bytes containing little-endian SPIR-V.
    pub fn from_bytes(
        device: &'device VulkanLogicalDevice,
        bytes: &[u8],
    ) -> Result<Self, VulkanShaderError> {
        if bytes.is_empty() {
            return Err(VulkanShaderError::EmptyCode);
        }
        if bytes.len() % std::mem::size_of::<u32>() != 0 {
            return Err(VulkanShaderError::InvalidByteLength);
        }

        let words = bytes.chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect::<Vec<_>>();

        Self::from_words(device, &words)
    }

    /// Validates the minimum SPIR-V container invariants without a GPU.
    pub fn validate_words(words: &[u32]) -> Result<(), VulkanShaderError> {
        if words.is_empty() {
            return Err(VulkanShaderError::EmptyCode);
        }
        if words[0] != 0x0723_0203 {
            return Err(VulkanShaderError::InvalidMagic);
        }
        if words.len() < 5 {
            return Err(VulkanShaderError::InvalidHeader);
        }
        Ok(())
    }

    pub fn handle(&self) -> vk::ShaderModule { self.handle }

    pub(super) fn device_handle(&self) -> vk::Device {
        self.device.handle().handle()
    }
}

impl Drop for VulkanShaderModule<'_> {
    fn drop(&mut self) {
        unsafe {
            self.device.handle().destroy_shader_module(self.handle, None);
        }
    }
}
