// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Compatibility re-exports for the original keyboard API path.
//!
//! New code should use the grouped input namespace.

pub use crate::input::keyboard::*;
pub use crate::input::{
    ExtensionCapability, ExtensionPermission, InputAction as KeyboardAction,
    InputDeviceCapabilities, InputDeviceDescriptor, InputDeviceId, InputDeviceKind,
    PeripheralExtensionDescriptor, StandardInputCapability,
};
