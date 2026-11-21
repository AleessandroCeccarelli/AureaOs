// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Coordinated pointing-device services.

mod applied_settings;
mod device_settings;
mod discovery;
mod mouse;
mod registry;
mod tablet;
mod tablet_calibration;
mod tablet_outputs;
mod touchpad;
mod touchpad_settings;
mod touchscreen;
mod touchscreen_calibration;
mod touchscreen_slots;
pub mod touchscreen_transform;

pub use applied_settings::AppliedTouchpadSettings;
pub use device_settings::PointingDeviceSettings;
pub(crate) use discovery::{describe, id_for};
pub use registry::PointingDeviceService;
pub use tablet_outputs::{TabletOutputMap, TabletOutputTarget};
pub use touchscreen::TouchscreenOutputMap;
pub use touchscreen_calibration::{
    TouchscreenCalibration, TouchscreenCalibrationError, TouchscreenCalibrationMap,
};
pub use touchscreen_slots::TouchscreenSlotMap;

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

pub use tablet_calibration::{
    TabletAspectMode, TabletCalibration, TabletCalibrationError, TabletCalibrationMap,
    TabletRotation,
};
