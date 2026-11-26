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
mod pointer_controller;
mod desktop_pointer_behavior;

pub use action::InputAction;
pub use device::{
    InputDeviceCapabilities, InputDeviceDescriptor, InputDeviceEvent, InputDeviceId,
    InputDeviceKind, InputDeviceProvider, StandardInputCapability, TouchpadCapabilities,
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
pub use pointer_controller::{PointerSettingsCommand, PointerSettingsController, pointer_settings_channel};
pub use desktop_pointer_behavior::{
    DesktopPointerBehaviorCommand, DesktopPointerBehaviorController,
    DesktopPointerBehaviorSettings, desktop_pointer_behavior_channel,
};
pub use touchpad_controller::{
    TouchpadSettingsCommand, TouchpadSettingsController, touchpad_settings_channel,
};
