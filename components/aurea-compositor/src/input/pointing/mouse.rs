// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Mouse-class device detection.

use smithay::reexports::input::{Device, DeviceCapability};

pub(super) fn supports(device: &Device) -> bool {
    device.has_capability(DeviceCapability::Pointer)
        && !device.has_capability(DeviceCapability::Touch)
        && !device.has_capability(DeviceCapability::TabletTool)
        && !device.has_capability(DeviceCapability::TabletPad)
        && device.config_tap_finger_count() == 0
        && !device.config_dwt_is_available()
}
