// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Translate libinput device metadata into AureaOS pointing-device descriptors.

use sha2::{Digest, Sha256};
use std::{fs, path::Path};

use aurea_desktop_api::input::{
    InputDeviceCapabilities, InputDeviceDescriptor, InputDeviceId, InputDeviceKind,
    StandardInputCapability,
};
use smithay::reexports::input::{Device, DeviceCapability};

use super::{mouse, tablet, touchpad, touchscreen};

/// Builds a public descriptor for a supported pointing device.
///
/// Touchpads are identified through libinput's touchpad-specific configuration
/// capabilities, not by guessing from the product name. Tablet tools take
/// precedence over pointer and touch capabilities.
pub(crate) fn id_for(device: &Device) -> InputDeviceId {
    InputDeviceId(stable_device_id(device.sysname()))
}

pub(crate) fn describe(device: &Device) -> Option<InputDeviceDescriptor> {
    let pointer = device.has_capability(DeviceCapability::Pointer);
    let touch = device.has_capability(DeviceCapability::Touch);
    let kind = if tablet::supports(device) {
        InputDeviceKind::Tablet
    } else if tablet::supports_pad(device) {
        InputDeviceKind::TabletPad
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

        if device.pointer_has_button(0x112).unwrap_or(false) {
            capabilities.push(StandardInputCapability::PointerMiddleButton);
        }
        // Linux input codes: BTN_SIDE, BTN_EXTRA, BTN_FORWARD, BTN_BACK.
        // Report navigation buttons as part of the side-button capability too.
        if has_pointer_side_buttons(|code| device.pointer_has_button(code).unwrap_or(false)) {
            capabilities.push(StandardInputCapability::PointerSideButtons);
        }
    }
    if touch {
        capabilities.push(StandardInputCapability::Touch);
    }
    if device.has_capability(DeviceCapability::Gesture) {
        capabilities.push(StandardInputCapability::Gestures);
    }
    if device.has_capability(DeviceCapability::TabletPad) {
        if device.tablet_pad_number_of_buttons() > 0 {
            capabilities.push(StandardInputCapability::TabletPadButtons);
        }
        if device.tablet_pad_number_of_rings() > 0 {
            capabilities.push(StandardInputCapability::TabletPadRings);
        }
        if device.tablet_pad_number_of_strips() > 0 {
            capabilities.push(StandardInputCapability::TabletPadStrips);
        }
    }

    Some(InputDeviceDescriptor {
        id: id_for(device),
        name: device.name().to_string(),
        vendor_id: nonzero_id(device.id_vendor()),
        product_id: nonzero_id(device.id_product()),
        persistent_id: stable_hardware_identity(device),
        kind,
        capabilities: InputDeviceCapabilities::new(capabilities),
    })
}

/// Checks Linux auxiliary and navigation button codes without assuming a layout.
fn has_pointer_side_buttons(mut supports: impl FnMut(u32) -> bool) -> bool {
    // BTN_SIDE, BTN_EXTRA, BTN_FORWARD, BTN_BACK.
    [0x113, 0x114, 0x115, 0x116].into_iter().any(&mut supports)
}

fn nonzero_id(value: u32) -> Option<u16> {
    (value != 0).then_some(value as u16)
}

/// Returns a stable hardware identity without reading or storing serial numbers.
fn stable_hardware_identity(device: &Device) -> Option<String> {
    #[cfg(feature = "drm")]
    let physical_path = udev::Device::from_subsystem_sysname(
        "input".to_string(),
        device.sysname().to_string(),
    )
    .ok()
    .and_then(|udev_device| {
        udev_device
            .property_value("ID_PATH")
            .map(|value| value.to_string_lossy().into_owned())
    });

    #[cfg(not(feature = "drm"))]
    let physical_path: Option<String> = None;

    let physical_path = physical_path.or_else(|| {
        let event_path = Path::new("/sys/class/input")
            .join(device.sysname())
            .join("device");
        let resolved = fs::canonicalize(event_path).ok()?;
        let physical = resolved.parent()?.parent()?;
        Some(physical.to_string_lossy().into_owned())
    })?;

    Some(stable_hardware_identity_from_parts(
        device.id_vendor(),
        device.id_product(),
        physical_path.as_bytes(),
    ))
}


/// Hashes stable hardware metadata and the physical sysfs path. The kernel
/// event number is deliberately excluded so profiles survive reboot/hotplug.
fn stable_hardware_identity_from_parts(
    vendor: u32,
    product: u32,
    physical_path: &[u8],
) -> String {
    let mut digest = Sha256::new();
    digest.update(vendor.to_le_bytes());
    digest.update(product.to_le_bytes());
    digest.update(physical_path);
    let hash = digest.finalize();
    format!(
        "libinput:{vendor:04x}:{product:04x}:{}",
        hash.iter().map(|byte| format!("{byte:02x}")).collect::<String>()
    )
}

/// Hashes the current kernel input sysname into a deterministic session ID.
///
/// `/dev/input/eventN` numbering may change across boots or reconnects, so this
/// value must not be used as a persistent physical-device identifier.
fn stable_device_id(value: &str) -> u64 {
    // FNV-1a: deterministic across runs, unlike DefaultHasher.
    value.bytes().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}

#[cfg(test)]
mod tests {
    use super::{has_pointer_side_buttons, stable_device_id, stable_hardware_identity_from_parts};

    #[test]
    fn recognizes_each_standard_auxiliary_or_navigation_button() {
        for expected in [0x113, 0x114, 0x115, 0x116] {
            assert!(has_pointer_side_buttons(|code| code == expected));
        }
    }

    #[test]
    fn does_not_report_side_buttons_when_none_are_present() {
        assert!(!has_pointer_side_buttons(|_| false));
    }

    #[test]
    fn hardware_identity_ignores_event_number_but_tracks_physical_path() {
        let first = stable_hardware_identity_from_parts(
            0x06cb,
            0x00f9,
            b"/sys/devices/platform/i2c-ELAN/input",
        );
        let reconnected = stable_hardware_identity_from_parts(
            0x06cb,
            0x00f9,
            "Synaptics Touchpad",
            b"/sys/devices/platform/i2c-ELAN/input",
        );
        let other_port = stable_hardware_identity_from_parts(
            0x06cb,
            0x00f9,
            b"/sys/devices/platform/i2c-ELAN/input-other",
        );
        assert_eq!(first, reconnected);
        assert_ne!(first, other_port);
        assert!(!first.contains("event"));
    }

    #[test]
    fn device_id_is_stable_for_the_same_sysname() {
        let first = stable_device_id("event4");
        let second = stable_device_id("event4");
        assert_eq!(first, second);
    }

    #[test]
    fn different_sysnames_produce_different_device_ids() {
        assert_ne!(stable_device_id("event4"), stable_device_id("event5"));
    }
}
