// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use smithay::reexports::ash::vk;

use aurea_compositor::vulkan::select_memory_type_index;

#[test]
fn selects_preferred_compatible_memory_type() {
    let mut properties = vk::PhysicalDeviceMemoryProperties::default();
    properties.memory_type_count = 3;
    properties.memory_types[0] = vk::MemoryType {
        property_flags: vk::MemoryPropertyFlags::HOST_VISIBLE,
        heap_index: 0,
    };
    properties.memory_types[1] = vk::MemoryType {
        property_flags: vk::MemoryPropertyFlags::DEVICE_LOCAL,
        heap_index: 0,
    };
    properties.memory_types[2] = vk::MemoryType {
        property_flags: vk::MemoryPropertyFlags::DEVICE_LOCAL
            | vk::MemoryPropertyFlags::HOST_VISIBLE,
        heap_index: 0,
    };

    assert_eq!(
        select_memory_type_index(
            &properties,
            0b110,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::MemoryPropertyFlags::HOST_VISIBLE,
        ),
        Some(2),
    );
}

#[test]
fn falls_back_to_required_flags_when_preference_is_unavailable() {
    let mut properties = vk::PhysicalDeviceMemoryProperties::default();
    properties.memory_type_count = 1;
    properties.memory_types[0].property_flags = vk::MemoryPropertyFlags::DEVICE_LOCAL;

    assert_eq!(
        select_memory_type_for_test(
            &properties,
            0b1,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::MemoryPropertyFlags::HOST_VISIBLE,
        ),
        Some(0),
    );
}

#[test]
fn rejects_memory_types_outside_requirement_bitmask() {
    let mut properties = vk::PhysicalDeviceMemoryProperties::default();
    properties.memory_type_count = 1;
    properties.memory_types[0].property_flags = vk::MemoryPropertyFlags::DEVICE_LOCAL;

    assert_eq!(
        select_memory_type_for_test(
            &properties,
            0,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        ),
        None,
    );
}
