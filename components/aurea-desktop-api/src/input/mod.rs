// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Technology-independent input contracts shared by AureaOS components.

mod action;
mod device;
mod extension;
pub mod keyboard;
pub mod pointer;
pub mod touchpad;
mod touchpad_controller;

pub use action::InputAction;
pub use device::{
    InputDeviceCapabilities, InputDeviceDescriptor, InputDeviceEvent, InputDeviceId,
    InputDeviceKind, InputDeviceProvider, StandardInputCapability,
};
pub use extension::{ExtensionCapability, ExtensionPermission, PeripheralExtensionDescriptor};
pub use pointer::{
    PointerAccelerationProfile, PointerButton, PointerSettings, PointerSettingsError,
    PointerSettingsProvider,
};
pub use touchpad::{
    TouchpadAccelerationProfile, TouchpadApplyReport, TouchpadClickMethod,
    TouchpadSendEventsMode, TouchpadSettingStatus,
    TouchpadSettings, TouchpadTapButtonMap,
};
pub use touchpad_controller::{
    TouchpadSettingsCommand, TouchpadSettingsController, touchpad_settings_channel,
};
