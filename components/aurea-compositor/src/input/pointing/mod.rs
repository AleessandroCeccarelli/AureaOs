// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Coordinated pointing-device services.

mod absolute_pointer_outputs;
mod applied_settings;
mod device_settings;
mod desktop_pointer_behavior_store;
mod discovery;
mod manager;
mod mouse;
mod pointer_store;
mod registry;
mod tablet;
mod tablet_tools;
mod tablet_store;
mod tablet_calibration;
mod tablet_outputs;
mod touchpad;
mod touchpad_settings;
mod touchpad_store;
mod touchscreen;
mod touchscreen_calibration;
mod touchscreen_slots;
pub mod touchscreen_transform;

pub use absolute_pointer_outputs::AbsolutePointerOutputMap;
pub use applied_settings::AppliedTouchpadSettings;
pub use device_settings::PointingDeviceSettings;
pub use desktop_pointer_behavior_store::DesktopPointerBehaviorStore;
pub(crate) use discovery::{describe, id_for};
pub use manager::PointerManager;
pub use pointer_store::PointerProfileStore;
pub use registry::PointingDeviceService;
pub use tablet_outputs::{TabletOutputMap, TabletOutputTarget};
pub use tablet_tools::{RemovedTabletTool, TabletToolRegistry};
pub use tablet_store::TabletProfileStore;
pub use touchscreen::TouchscreenOutputMap;
pub use touchscreen_calibration::{
    TouchscreenCalibration, TouchscreenCalibrationError, TouchscreenCalibrationMap,
};
pub use touchscreen_slots::TouchscreenSlotMap;

/// Applies AureaOS defaults to a newly connected touchpad.
pub(crate) fn configure_touchpad(
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
