// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Deterministic Vulkan memory-type selection.

use smithay::reexports::ash::vk;

/// Selects a memory type satisfying all required flags, preferring optional flags.
///
/// Returns the first preferred match. If no preferred match exists, returns
/// the first type satisfying the required flags. A missing compatible type
/// is reported explicitly; callers must not silently weaken required flags.
pub fn select_memory_type_index(
    properties: &vk::PhysicalDeviceMemoryProperties,
    memory_type_bits: u32,
    required: vk::MemoryPropertyFlags,
    preferred: vk::MemoryPropertyFlags,
) -> Option<u32> {
    let compatible = properties.memory_types[..properties.memory_type_count as usize]
        .iter()
        .enumerate()
        .filter(|(index, memory_type)| {
            (memory_type_bits & (1 << index)) != 0
                && memory_type.property_flags.contains(required)
        });

    let candidates = compatible.collect::<Vec<_>>();

    candidates
        .iter()
        .find(|(_, memory_type)| memory_type.property_flags.contains(preferred))
        .or_else(|| candidates.first())
        .map(|(index, _)| *index as u32)
}
