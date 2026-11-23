// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Shared hardware monitoring contracts.
//!
//! This module starts with the inventory coordinator. Device-specific
//! providers and managers are added here incrementally as the desktop grows.

mod coordinator;

pub use coordinator::{
    HardwareConnectionState, HardwareDeviceId, HardwareDeviceKind, HardwareDeviceRecord,
    HardwareDeviceState, HardwareMonitorCoordinator, HardwareMonitorModule,
    HardwareMonitorSnapshot,
};
