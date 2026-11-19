// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Central inventory and lifecycle state for hardware known to AureaOS.
//!
//! This coordinator is deliberately backend-agnostic. It does not discover
//! devices, open them, or own driver resources; dedicated managers will do
//! that and report their observations here.

use std::collections::{BTreeMap, BTreeSet};

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

/// Hardware-monitor service modules currently supported by AureaOS.
///
/// New modules are introduced deliberately as their contracts and managers
/// are implemented; future categories are not exposed prematurely.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HardwareMonitorModule {
    Keyboard,
    Mouse,
    Monitor,
}

/// Hardware category represented in the shared inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareDeviceKind {
    Keyboard,
    Mouse,
    Monitor,
}

/// Physical connection state reported by the platform device monitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareConnectionState {
    Connected,
    Disconnected,
}

/// Operational condition reported by a device-specific manager.
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
    pub connection: HardwareConnectionState,
    pub state: HardwareDeviceState,
}

/// Consistent point-in-time view of the coordinator inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardwareMonitorSnapshot {
    pub revision: u64,
    pub modules: Vec<HardwareMonitorModule>,
    pub devices: Vec<HardwareDeviceRecord>,
}

/// Central registry updated by the future device-specific managers.
///
/// A disconnected device remains in the inventory so that higher-level
/// services can retain preferences associated with its stable identifier.
#[derive(Debug)]
pub struct HardwareMonitorCoordinator {
    revision: u64,
    modules: BTreeSet<HardwareMonitorModule>,
    devices: BTreeMap<HardwareDeviceId, HardwareDeviceRecord>,
}

impl Default for HardwareMonitorCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl HardwareMonitorCoordinator {
    /// Creates an empty hardware inventory.
    pub fn new() -> Self {
        Self {
            revision: 0,
            modules: [
                HardwareMonitorModule::Keyboard,
                HardwareMonitorModule::Mouse,
                HardwareMonitorModule::Monitor,
            ]
            .into_iter()
            .collect(),
            devices: BTreeMap::new(),
        }
    }

    /// Registers a supported service module. Returns false if already present.
    pub fn register_module(&mut self, module: HardwareMonitorModule) -> bool {
        if !self.modules.insert(module) {
            return false;
        }

        self.advance_revision();
        true
    }

    /// Unregisters a service module without deleting its remembered devices.
    pub fn unregister_module(&mut self, module: HardwareMonitorModule) -> bool {
        if !self.modules.remove(&module) {
            return false;
        }

        self.advance_revision();
        true
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

    /// Updates the physical connection state without deleting device identity.
    pub fn set_connection_state(
        &mut self,
        id: &HardwareDeviceId,
        connection: HardwareConnectionState,
    ) -> bool {
        let Some(record) = self.devices.get_mut(id) else {
            return false;
        };

        if record.connection == connection {
            return false;
        }

        record.connection = connection;
        self.advance_revision();
        true
    }

    /// Marks a device as physically disconnected without deleting its identity.
    pub fn disconnect_device(&mut self, id: &HardwareDeviceId) -> bool {
        self.set_connection_state(id, HardwareConnectionState::Disconnected)
    }

    /// Returns whether a known device is physically connected.
    pub fn is_device_connected(&self, id: &HardwareDeviceId) -> bool {
        self.devices
            .get(id)
            .is_some_and(|record| record.connection == HardwareConnectionState::Connected)
    }

    /// Returns only devices that are physically connected.
    pub fn connected_devices(&self) -> Vec<HardwareDeviceRecord> {
        self.devices
            .values()
            .filter(|record| record.connection == HardwareConnectionState::Connected)
            .cloned()
            .collect()
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
            modules: self.modules.iter().copied().collect(),
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
        HardwareConnectionState, HardwareDeviceId, HardwareDeviceKind, HardwareDeviceRecord,
        HardwareDeviceState, HardwareMonitorCoordinator, HardwareMonitorModule,
    };

    fn keyboard(id: &str, state: HardwareDeviceState) -> HardwareDeviceRecord {
        HardwareDeviceRecord {
            id: HardwareDeviceId::new(id).expect("test identifier must be valid"),
            kind: HardwareDeviceKind::Keyboard,
            name: "Keyboard".to_owned(),
            connection: HardwareConnectionState::Connected,
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
        assert_eq!(snapshot.devices[0].connection, HardwareConnectionState::Disconnected);
        assert!(coordinator.connected_devices().is_empty());
        assert!(!coordinator.is_device_connected(&id));

        assert!(coordinator.set_connection_state(&id, HardwareConnectionState::Connected));
        assert!(coordinator.is_device_connected(&id));
        assert_eq!(coordinator.connected_devices().len(), 1);
    }

    #[test]
    fn connection_and_operational_state_are_independent() {
        let mut coordinator = HardwareMonitorCoordinator::new();
        let id = HardwareDeviceId::new("usb:keyboard-1").unwrap();
        coordinator.upsert_device(keyboard("usb:keyboard-1", HardwareDeviceState::Ready));

        assert!(coordinator.is_device_connected(&id));
        assert!(coordinator.set_device_state(&id, HardwareDeviceState::Unavailable));
        assert!(coordinator.is_device_connected(&id));
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
    fn only_initial_hardware_modules_are_registered() {
        let coordinator = HardwareMonitorCoordinator::new();
        assert_eq!(
            coordinator.snapshot().modules,
            vec![
                HardwareMonitorModule::Keyboard,
                HardwareMonitorModule::Mouse,
                HardwareMonitorModule::Monitor,
            ]
        );
    }

    #[test]
    fn module_registration_is_idempotent_and_keeps_inventory_separate() {
        let mut coordinator = HardwareMonitorCoordinator::new();
        assert!(!coordinator.register_module(HardwareMonitorModule::Mouse));
        assert!(coordinator.unregister_module(HardwareMonitorModule::Mouse));
        assert!(!coordinator.snapshot().modules.contains(&HardwareMonitorModule::Mouse));
        assert!(coordinator.register_module(HardwareMonitorModule::Mouse));
    }

    #[test]
    fn empty_device_identifiers_are_rejected() {
        assert!(HardwareDeviceId::new("").is_none());
        assert!(HardwareDeviceId::new("   ").is_none());
    }
}
