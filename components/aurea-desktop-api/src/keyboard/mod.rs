// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Shared keyboard, layout and peripheral-extension contracts.

mod action;
mod device;
mod extension;
mod layout;

pub use action::KeyboardAction;
pub use device::{
    InputDeviceCapabilities, InputDeviceId, InputDeviceKind, StandardInputCapability,
};
pub use extension::{
    ExtensionCapability, ExtensionPermission, PeripheralExtensionDescriptor,
};
pub use layout::{
    KeyboardLayoutError, KeyboardLayoutHintSource, KeyboardLayoutId, KeyboardLayoutInfo,
    KeyboardLayoutManager, KeyboardLayoutSelection, KeyboardLayoutSuggestion,
};
