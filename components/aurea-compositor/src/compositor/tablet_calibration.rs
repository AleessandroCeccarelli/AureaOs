// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Compositor API for per-device tablet calibration.

use aurea_desktop_api::input::{InputDeviceId, InputDeviceKind, InputDeviceProvider};

use crate::input::pointing::{
    TabletAspectMode, TabletCalibration, TabletCalibrationError, TabletRotation,
};

use super::AureaCompositor;

impl AureaCompositor {
    /// Sets a validated calibration profile for a connected pen tablet.
    pub fn set_tablet_calibration(
        &mut self,
        device: InputDeviceId,
        active_area: [f64; 4],
        rotation: TabletRotation,
        aspect_mode: TabletAspectMode,
        device_aspect_ratio: f64,
    ) -> Result<(), TabletCalibrationError> {
        if !self.pointing_devices.device(device).is_some_and(|descriptor| {
            descriptor.kind == InputDeviceKind::Tablet
        }) {
            return Err(TabletCalibrationError::UnknownDevice);
        }

        let profile = TabletCalibration::new(
            active_area, rotation, aspect_mode, device_aspect_ratio,
        )?;
        self.tablet_calibrations.set(device, profile);
        Ok(())
    }

    /// Restores identity mapping for one connected tablet.
    pub fn reset_tablet_calibration(&mut self, device: InputDeviceId) -> bool {
        self.tablet_calibrations.remove_device(device)
    }

    /// Returns the configured tablet calibration, or identity by default.
    pub(crate) fn tablet_calibration(&self, device: InputDeviceId) -> TabletCalibration {
        self.tablet_calibrations.get(device)
    }
}
