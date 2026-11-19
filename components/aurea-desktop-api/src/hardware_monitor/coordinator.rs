// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Central inventory and lifecycle state for hardware known to AureaOS.
//!
//! This coordinator is deliberately backend-agnostic. It does not discover
//! devices, open them, or own driver resources; dedicated managers will do
//! that and report their observations here.

use std::collections::BTreeMap;

/// Stable identifier supplied by the subsystem responsible for a device.
///
/// The value is opaque to the coordinator. Providers should prefer persistent
/// hardware identity over a transient connection or enumeration number.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HardwareDeviceId(String);

impl HardwareDeviceId {
    /// Creates an identifier, rejecting empty or whitespace-only values.
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return None;
        }

        Some(Self(value))
    }

    /// Returns the opaque identifier value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Broad hardware category understood by the shared coordinator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareDeviceKind {
    Keyboard,
    Pointer,
    Display,
    Camera,
    Printer,
    Audio,
    Storage,
    Other,
}

/// Lifecycle and operational condition reported by a device manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareDeviceState {
    Discovered,
    Connected,
    Ready,
    Degraded,
    Unavailable,
    Disconnected,
}

/// Backend-independent summary of one hardware device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardwareDeviceRecord {
    pub id: HardwareDeviceId,
    pub kind: HardwareDeviceKind,
    pub name: String,
    pub state: HardwareDeviceState,
}

/// Consistent point-in-time view of the coordinator inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardwareMonitorSnapshot {
    pub revision: u64,
    pub devices: Vec<HardwareDeviceRecord>,
}

/// Central registry updated by the future device-specific managers.
///
/// A disconnected device remains in the inventory so that higher-level
/// services can retain preferences associated with its stable identifier.
#[derive(Debug, Default)]
pub struct HardwareMonitorCoordinator {
    revision: u64,
    devices: BTreeMap<HardwareDeviceId, HardwareDeviceRecord>,
}

impl HardwareMonitorCoordinator {
    /// Creates an empty hardware inventory.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a device or refreshes its reported information.
    ///
    /// Returns whether the inventory actually changed.
    pub fn upsert_device(&mut self, record: HardwareDeviceRecord) -> bool {
        if self.devices.get(&record.id) == Some(&record) {
            return false;
        }

        self.devices.insert(record.id.clone(), record);
        self.advance_revision();
        true
    }

    /// Updates the lifecycle state of a known device.
    ///
    /// Returns false when the identifier is unknown or the state is unchanged.
    pub fn set_device_state(
        &mut self,
        id: &HardwareDeviceId,
        state: HardwareDeviceState,
    ) -> bool {
        let Some(record) = self.devices.get_mut(id) else {
            return false;
        };

        if record.state == state {
            return false;
        }

        record.state = state;
        self.advance_revision();
        true
    }

    /// Marks a device as disconnected without deleting its identity.
    pub fn disconnect_device(&mut self, id: &HardwareDeviceId) -> bool {
        self.set_device_state(id, HardwareDeviceState::Disconnected)
    }

    /// Removes a device record explicitly, for example after an identity
    /// has been invalidated or an administrator requests inventory cleanup.
    pub fn remove_device(&mut self, id: &HardwareDeviceId) -> bool {
        if self.devices.remove(id).is_none() {
            return false;
        }

        self.advance_revision();
        true
    }

    /// Returns a stable, identifier-sorted inventory snapshot.
    pub fn snapshot(&self) -> HardwareMonitorSnapshot {
        HardwareMonitorSnapshot {
            revision: self.revision,
            devices: self.devices.values().cloned().collect(),
        }
    }

    fn advance_revision(&mut self) {
        self.revision = self.revision.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        HardwareDeviceId, HardwareDeviceKind, HardwareDeviceRecord, HardwareDeviceState,
        HardwareMonitorCoordinator,
    };

    fn keyboard(id: &str, state: HardwareDeviceState) -> HardwareDeviceRecord {
        HardwareDeviceRecord {
            id: HardwareDeviceId::new(id).expect("test identifier must be valid"),
            kind: HardwareDeviceKind::Keyboard,
            name: "Keyboard".to_owned(),
            state,
        }
    }

    #[test]
    fn disconnected_device_remains_in_inventory_for_reconnection() {
        let mut coordinator = HardwareMonitorCoordinator::new();
        let id = HardwareDeviceId::new("usb:keyboard-1").unwrap();

        coordinator.upsert_device(keyboard(
            "usb:keyboard-1",
            HardwareDeviceState::Ready,
        ));
        assert!(coordinator.disconnect_device(&id));

        let snapshot = coordinator.snapshot();
        assert_eq!(snapshot.devices.len(), 1);
        assert_eq!(snapshot.devices[0].state, HardwareDeviceState::Disconnected);

        assert!(coordinator.set_device_state(&id, HardwareDeviceState::Ready));
        assert_eq!(
            coordinator.snapshot().devices[0].state,
            HardwareDeviceState::Ready
        );
    }

    #[test]
    fn unchanged_updates_do_not_advance_revision() {
        let mut coordinator = HardwareMonitorCoordinator::new();
        let record = keyboard("usb:keyboard-1", HardwareDeviceState::Ready);

        assert!(coordinator.upsert_device(record.clone()));
        assert!(!coordinator.upsert_device(record));
        assert_eq!(coordinator.snapshot().revision, 1);
    }

    #[test]
    fn unknown_devices_cannot_be_updated() {
        let mut coordinator = HardwareMonitorCoordinator::new();
        let id = HardwareDeviceId::new("unknown").unwrap();

        assert!(!coordinator.set_device_state(&id, HardwareDeviceState::Ready));
        assert_eq!(coordinator.snapshot().revision, 0);
    }

    #[test]
    fn empty_device_identifiers_are_rejected() {
        assert!(HardwareDeviceId::new("").is_none());
        assert!(HardwareDeviceId::new("   ").is_none());
    }
}
