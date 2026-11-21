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

/// Builds a public descriptor for a supported pointing device.
///
/// Touchpads are identified through libinput's touchpad-specific configuration
/// capabilities, not by guessing from the product name. Tablet tools take
/// precedence over pointer and touch capabilities.
pub(super) fn describe(device: &Device) -> Option<InputDeviceDescriptor> {
    let pointer = device.has_capability(DeviceCapability::Pointer);
    let touch = device.has_capability(DeviceCapability::Touch);
    let tablet_tool = device.has_capability(DeviceCapability::TabletTool);
    let tablet_pad = device.has_capability(DeviceCapability::TabletPad);
    let touchpad = device.config_tap_finger_count() > 0 || device.config_dwt_is_available();

    let kind = if tablet_tool || tablet_pad {
        InputDeviceKind::Tablet
    } else if touchpad {
        InputDeviceKind::Touchpad
    } else if touch {
        InputDeviceKind::Touchscreen
    } else if pointer {
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
        id: InputDeviceId(stable_device_id(device.sysname())),
        name: device.name().to_string(),
        vendor_id: Some(device.id_vendor() as u16),
        product_id: Some(device.id_product() as u16),
        kind,
        capabilities: InputDeviceCapabilities::new(capabilities),
    })
}

fn stable_device_id(value: &str) -> u64 {
    // FNV-1a: deterministic across runs, unlike DefaultHasher.
    value.bytes().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}
