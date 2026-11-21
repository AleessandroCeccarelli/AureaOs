// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Touchpad-class device detection using libinput's supported settings.

use smithay::reexports::input::Device;

pub(super) fn supports(device: &Device) -> bool {
    device.config_tap_finger_count() > 0 || device.config_dwt_is_available()
}
