// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Translate backend device metadata into AureaOS pointing-device descriptors.

use aurea_desktop_api::input::{
    InputDeviceCapabilities, InputDeviceDescriptor, InputDeviceId, InputDeviceKind,
    StandardInputCapability,
};
use smithay::backend::input::{Device, DeviceCapability};

/// Builds a public descriptor for a supported pointing device.
///
/// Classification is capability-based. A device exposing both pointer and
/// touch capabilities is treated as a touchpad; touch-only devices are
/// treated as touchscreens. Tablet tools take precedence over both.
pub(super) fn describe<D: Device>(device: &D) -> Option<InputDeviceDescriptor> {
    let pointer = device.has_capability(DeviceCapability::Pointer);
    let touch = device.has_capability(DeviceCapability::Touch);
    let tablet_tool = device.has_capability(DeviceCapability::TabletTool);
    let tablet_pad = device.has_capability(DeviceCapability::TabletPad);

    let kind = if tablet_tool || tablet_pad {
        InputDeviceKind::Tablet
    } else if pointer && touch {
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

    let (vendor_id, product_id) = device
        .usb_id()
        .map(|(product, vendor)| (Some(vendor as u16), Some(product as u16)))
        .unwrap_or((None, None));

    Some(InputDeviceDescriptor {
        id: InputDeviceId(stable_device_id(&device.id())),
        kind,
        name: device.name(),
        vendor_id,
        product_id,
        capabilities: InputDeviceCapabilities::new(capabilities),
    })
}

fn stable_device_id(value: &str) -> u64 {
    // FNV-1a: deterministic across runs, unlike DefaultHasher.
    value.bytes().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}
