// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Direct-touch display device detection.

use smithay::reexports::input::{Device, DeviceCapability};

pub(super) fn supports(device: &Device) -> bool {
    device.has_capability(DeviceCapability::Touch)
        && !device.has_capability(DeviceCapability::TabletTool)
        && !device.has_capability(DeviceCapability::TabletPad)
}
