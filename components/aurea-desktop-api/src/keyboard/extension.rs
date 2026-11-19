// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Declarative, permissioned contract for optional peripheral extensions.
//!
//! This API describes extensions; it does not load or execute vendor code.
//! Runtime isolation and IPC are owned by a separate extension manager.

/// Optional operation a manufacturer extension may provide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtensionCapability {
    CustomButtonMapping,
    PerApplicationProfiles,
    GestureActions,
    DeviceLighting,
    BatteryInformation,
    HapticFeedback,
    FirmwareManagement,
}

/// Permission an extension must request before the system enables a capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtensionPermission {
    ConfigurePeripheral,
    ReadPeripheralStatus,
    ControlPeripheralFeedback,
    ManageFirmware,
    RegisterDesktopActions,
}

/// Public metadata declared by a vendor extension package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeripheralExtensionDescriptor {
    pub id: String,
    pub display_name: String,
    pub api_major: u16,
    pub api_minor: u16,
    pub supported_vendor_ids: Vec<u16>,
    pub capabilities: Vec<ExtensionCapability>,
    pub permissions: Vec<ExtensionPermission>,
}
