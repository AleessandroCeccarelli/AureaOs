// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Applies validated pointer preferences to one libinput device.

use smithay::reexports::input::{AccelProfile, Device};

use super::pointer_settings::{AccelerationProfile, PointerSettings};

/// Applies supported acceleration settings without resetting unchanged values.
pub fn apply(device: &mut Device, settings: PointerSettings) {
    if !device.config_accel_is_available() {
        return;
    }

    let requested_profile = match settings.profile() {
        AccelerationProfile::Adaptive => AccelProfile::Adaptive,
        AccelerationProfile::Flat => AccelProfile::Flat,
    };

    if device.config_accel_profiles().contains(&requested_profile)
        && device.config_accel_profile() != Some(requested_profile)
        && let Err(error) = device.config_accel_set_profile(requested_profile)
    {
        eprintln!(
            "AureaOS: could not set pointer acceleration profile for {}: {error:?}",
            device.name()
        );
    }

    if (device.config_accel_speed() - settings.speed()).abs() > f64::EPSILON
        && let Err(error) = device.config_accel_set_speed(settings.speed())
    {
        eprintln!(
            "AureaOS: could not set pointer acceleration speed for {}: {error:?}",
            device.name()
        );
    }
}
