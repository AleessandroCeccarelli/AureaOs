// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Coordinated pointing-device services.

mod applied_settings;
mod discovery;
mod device_settings;
mod mouse;
mod registry;
mod tablet;
mod tablet_outputs;
mod touchscreen;
mod touchscreen_calibration;
pub mod touchscreen_transform;
mod touchscreen_slots;
mod touchpad;
mod touchpad_settings;

pub use registry::PointingDeviceService;
pub use applied_settings::AppliedTouchpadSettings;
pub use device_settings::PointingDeviceSettings;
pub use touchscreen::TouchscreenOutputMap;
pub use touchscreen_calibration::{
    TouchscreenCalibration, TouchscreenCalibrationError, TouchscreenCalibrationMap,
};
pub use touchscreen_slots::TouchscreenSlotMap;
pub use tablet_outputs::{TabletOutputMap, TabletOutputTarget};
pub(super) use discovery::{describe, id_for};

/// Applies AureaOS defaults to a newly connected touchpad.
pub(super) fn configure_touchpad(
    device: &mut smithay::reexports::input::Device,
    settings: aurea_desktop_api::input::TouchpadSettings,
) -> Option<aurea_desktop_api::input::TouchpadApplyReport> {
    if !touchpad::supports(device) {
        return None;
    }

    touchpad_settings::apply(device, settings)
}
