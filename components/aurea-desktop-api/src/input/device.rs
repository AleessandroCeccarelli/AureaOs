// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Technology-independent input-device identity and capability descriptions.

use std::sync::mpsc::Receiver;

/// Stable identifier assigned by the AureaOS input-device discovery layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InputDeviceId(pub u64);

/// Broad class of input hardware.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputDeviceKind {
    Keyboard,
    Mouse,
    Touchpad,
    Tablet,
    Touchscreen,
    Composite,
}

/// Standard behavior actually exposed by a connected device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StandardInputCapability {
    Keyboard,
    ConsumerControls,
    PointerMotion,
    PointerButtons,
    PointerMiddleButton,
    PointerSideButtons,
    VerticalScroll,
    HorizontalScroll,
    HighResolutionScroll,
    Touch,
    Gestures,
    PenPressure,
    PenTilt,
    PenDistance,
    PenRotation,
    PenEraser,
    PenButtons,
    TabletPadButtons,
    TabletPadRings,
    TabletPadStrips,
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
        Self {
            capabilities: values,
        }
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

/// Public, non-sensitive description of a discovered input device.
///
/// Serial numbers and raw event streams are deliberately excluded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputDeviceDescriptor {
    pub id: InputDeviceId,
    pub kind: InputDeviceKind,
    pub name: String,
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
    pub capabilities: InputDeviceCapabilities,
}

/// Lifecycle changes visible to trusted device-management clients.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputDeviceEvent {
    Connected(InputDeviceDescriptor),
    Disconnected(InputDeviceId),
    CapabilitiesChanged(InputDeviceDescriptor),
}

/// Read-only inventory contract for system settings and trusted desktop services.
pub trait InputDeviceProvider: Send + Sync {
    fn devices(&self) -> Vec<InputDeviceDescriptor>;
    fn device(&self, id: InputDeviceId) -> Option<InputDeviceDescriptor>;
    fn subscribe(&self) -> Receiver<InputDeviceEvent>;
}

#[cfg(test)]
mod tests {
    use super::{InputDeviceCapabilities, StandardInputCapability};

    #[test]
    fn capability_snapshots_deduplicate_verified_features() {
        let capabilities = InputDeviceCapabilities::new([
            StandardInputCapability::PointerMotion,
            StandardInputCapability::PointerButtons,
            StandardInputCapability::PointerMotion,
        ]);

        assert!(capabilities.supports(StandardInputCapability::PointerMotion));
        assert_eq!(capabilities.iter().count(), 2);
        assert!(!capabilities.supports(StandardInputCapability::Keyboard));
    }
}
