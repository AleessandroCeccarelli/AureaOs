// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Technology-independent input-device identity and capability descriptions.

/// Stable identifier assigned by the AureaOS input-device discovery layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InputDeviceId(pub u64);

/// Broad class of input hardware.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputDeviceKind {
    Keyboard,
    PointingDevice,
    Composite,
}

/// Standard behavior actually exposed by a connected device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StandardInputCapability {
    Keyboard,
    ConsumerControls,
    PointerButtons,
    HorizontalScroll,
    HighResolutionScroll,
    KeyboardBacklight,
    BatteryStatus,
}

/// Snapshot of capabilities discovered from descriptors and supported interfaces.
///
/// This is not a prediction based on a product name or vendor identifier.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InputDeviceCapabilities {
    capabilities: Vec<StandardInputCapability>,
}

impl InputDeviceCapabilities {
    /// Creates a capability set from verified device capabilities.
    pub fn new(capabilities: impl IntoIterator<Item = StandardInputCapability>) -> Self {
        let mut values = Vec::new();
        for capability in capabilities {
            if !values.contains(&capability) {
                values.push(capability);
            }
        }
        Self { capabilities: values }
    }

    /// Returns true only for a capability discovered on this device.
    pub fn supports(&self, capability: StandardInputCapability) -> bool {
        self.capabilities.contains(&capability)
    }

    /// Iterates over the capabilities discovered for this device.
    pub fn iter(&self) -> impl Iterator<Item = StandardInputCapability> + '_ {
        self.capabilities.iter().copied()
    }
}
