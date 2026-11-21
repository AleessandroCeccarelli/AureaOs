// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Translate libinput device metadata into AureaOS pointing-device descriptors.

use aurea_desktop_api::input::{
    InputDeviceCapabilities, InputDeviceDescriptor, InputDeviceId, InputDeviceKind,
    StandardInputCapability,
};
use smithay::reexports::input::{Device, DeviceCapability};

use super::{mouse, tablet, touchscreen, touchpad};

/// Builds a public descriptor for a supported pointing device.
///
/// Touchpads are identified through libinput's touchpad-specific configuration
/// capabilities, not by guessing from the product name. Tablet tools take
/// precedence over pointer and touch capabilities.
pub(super) fn id_for(device: &Device) -> InputDeviceId {
    InputDeviceId(stable_device_id(device.sysname()))
}

pub(super) fn describe(device: &Device) -> Option<InputDeviceDescriptor> {
    let pointer = device.has_capability(DeviceCapability::Pointer);
    let touch = device.has_capability(DeviceCapability::Touch);
    let kind = if tablet::supports(device) {
        InputDeviceKind::Tablet
    } else if touchpad::supports(device) {
        InputDeviceKind::Touchpad
    } else if touchscreen::supports(device) {
        InputDeviceKind::Touchscreen
    } else if mouse::supports(device) {
        InputDeviceKind::Mouse
    } else {
        return None;
    };

    let mut capabilities = Vec::new();
    if pointer {
        capabilities.push(StandardInputCapability::PointerMotion);
        capabilities.push(StandardInputCapability::PointerButtons);
    }
    if touch {
        capabilities.push(StandardInputCapability::Touch);
    }
    if device.has_capability(DeviceCapability::Gesture) {
        capabilities.push(StandardInputCapability::Gestures);
    }

    Some(InputDeviceDescriptor {
        id: id_for(device),
        name: device.name().to_string(),
        vendor_id: nonzero_id(device.id_vendor()),
        product_id: nonzero_id(device.id_product()),
        kind,
        capabilities: InputDeviceCapabilities::new(capabilities),
    })
}

fn nonzero_id(value: u32) -> Option<u16> {
    (value != 0).then_some(value as u16)
}

fn stable_device_id(value: &str) -> u64 {
    // FNV-1a: deterministic across runs, unlike DefaultHasher.
    value.bytes().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}
