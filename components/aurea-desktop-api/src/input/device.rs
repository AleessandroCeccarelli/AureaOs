// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Technology-independent input-device identity and capability descriptions.

use std::sync::mpsc::Receiver;

use super::{TouchpadApplyReport, TouchpadAccelerationProfile};

/// Session-scoped identifier assigned by the AureaOS input-device discovery layer.
///
/// The current libinput backend derives it from the kernel input sysname. It is
/// deterministic for that sysname, but is not a persistent hardware identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InputDeviceId(pub u64);

/// Broad class of input hardware.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputDeviceKind {
    Keyboard,
    Mouse,
    Touchpad,
    Tablet,
    TabletPad,
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

/// Detailed configuration capabilities exposed by one touchpad through libinput.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TouchpadCapabilities {
    pub max_tap_fingers: u32,
    pub tap_to_click: bool,
    pub tap_and_drag: bool,
    pub tap_drag_lock: bool,
    pub tap_button_map_configurable: bool,
    pub two_finger_scroll: bool,
    pub edge_scrolling: bool,
    pub natural_scrolling: bool,
    pub button_areas: bool,
    pub clickfinger: bool,
    pub disable_while_typing: bool,
    pub disable_while_trackpointing: bool,
    pub gestures: bool,
    pub left_handed: bool,
    pub middle_button_emulation: bool,
    pub pointer_acceleration: bool,
    pub flat_acceleration: bool,
    pub adaptive_acceleration: bool,
    pub disable_events: bool,
    pub disable_on_external_mouse: bool,
}

/// Snapshot of capabilities discovered from descriptors and supported interfaces.
///
/// This is not a prediction based on a product name or vendor identifier.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InputDeviceCapabilities {
    capabilities: Vec<StandardInputCapability>,
    touchpad: Option<TouchpadCapabilities>,
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
            touchpad: None,
        }
    }

    /// Attaches detailed touchpad capabilities to this snapshot.
    pub fn with_touchpad(mut self, capabilities: TouchpadCapabilities) -> Self {
        self.touchpad = Some(capabilities);
        self
    }

    /// Returns detailed touchpad capabilities, if this is a touchpad.
    pub fn touchpad(&self) -> Option<&TouchpadCapabilities> {
        self.touchpad.as_ref()
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
    /// Stable, privacy-preserving physical identity for persistent preferences.
    /// This is absent when the backend cannot establish a reliable identity.
    pub persistent_id: Option<String>,
    pub capabilities: InputDeviceCapabilities,
}

/// Lifecycle changes visible to trusted device-management clients.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputDeviceEvent {
    Connected(InputDeviceDescriptor),
    Disconnected(InputDeviceId),
    CapabilitiesChanged(InputDeviceDescriptor),
    TouchpadConfigurationChanged(InputDeviceId, TouchpadApplyReport),
}

/// Read-only inventory contract for system settings and trusted desktop services.
pub trait InputDeviceProvider: Send + Sync {
    fn devices(&self) -> Vec<InputDeviceDescriptor>;
    fn device(&self, id: InputDeviceId) -> Option<InputDeviceDescriptor>;
    /// Returns the latest hardware-specific touchpad configuration result.
    fn touchpad_report(&self, _id: InputDeviceId) -> Option<TouchpadApplyReport> {
        None
    }
    fn subscribe(&self) -> Receiver<InputDeviceEvent>;
}

#[cfg(test)]
mod tests {
    use super::{InputDeviceCapabilities, StandardInputCapability, TouchpadCapabilities};

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
        assert!(capabilities.touchpad().is_none());
    }

    #[test]
    fn detailed_touchpad_capabilities_are_optional_and_preserved() {
        let details = TouchpadCapabilities {
            max_tap_fingers: 3,
            two_finger_scroll: true,
            natural_scrolling: true,
            gestures: true,
            ..TouchpadCapabilities::default()
        };
        let capabilities = InputDeviceCapabilities::default().with_touchpad(details.clone());
        assert_eq!(capabilities.touchpad(), Some(&details));
    }
}
