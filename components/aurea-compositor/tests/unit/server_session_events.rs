// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS test module relocated from production source.

use super::*;
use crate::drm::{
    device::DrmDeviceError,
    device_manager::{DrmDeviceManager, DrmDeviceManagerError, ManagedDrmDevice},
};

#[derive(Debug)]
struct FakeDevice {
    node: u32,
    pauses: usize,
    activations: usize,
    fail_activation: bool,
}

impl ManagedDrmDevice for FakeDevice {
    type Identity = u32;
    fn identity(&self) -> Result<Self::Identity, DrmDeviceManagerError> {
        Ok(self.node)
    }

    fn pause(&mut self) {
        self.pauses += 1;
    }

    fn activate(&mut self) -> Result<(), DrmDeviceError> {
        self.activations += 1;
        if self.fail_activation {
            Err(DrmDeviceError::ActivationFailed(format!(
                "activation failed for {}",
                self.node
            )))
        } else {
            Ok(())
        }
    }
}

fn node(minor: u32) -> u32 {
    minor
}

fn fake_device(minor: u32) -> FakeDevice {
    FakeDevice {
        node: node(minor),
        pauses: 0,
        activations: 0,
        fail_activation: false,
    }
}

fn manager_with_two() -> DrmDeviceManager<FakeDevice, u32> {
    let mut devices = DrmDeviceManager::<FakeDevice, u32>::new();
    devices.register(fake_device(1)).unwrap();
    devices.register(fake_device(2)).unwrap();
    devices
}

#[test]
fn pause_event_updates_session_state() {
    let mut active = true;
    let mut devices = manager_with_two();
    let result = apply_session_event(SessionEvent::PauseSession, &mut active, &mut devices);

    assert!(result.is_ok());
    assert!(!active);
    assert_eq!(devices.get(node(1)).unwrap().pauses, 1);
    assert_eq!(devices.get(node(2)).unwrap().pauses, 1);
}

#[test]
fn activate_event_updates_session_state() {
    let mut active = false;

    let mut devices = manager_with_two();
    let result = apply_session_event(SessionEvent::ActivateSession, &mut active, &mut devices);

    assert!(result.is_ok());
    assert!(active);
    assert_eq!(devices.get(node(1)).unwrap().activations, 1);
    assert_eq!(devices.get(node(2)).unwrap().activations, 1);
}

#[test]
fn activation_attempts_all_devices_and_reports_partial_failure() {
    let mut active = false;
    let mut devices = manager_with_two();
    devices.get_mut(node(2)).unwrap().fail_activation = true;

    let result = apply_session_event(SessionEvent::ActivateSession, &mut active, &mut devices);

    assert!(matches!(result, Err(DrmDeviceError::ActivationFailed(_))));
    assert!(!active);
    assert_eq!(devices.get(node(1)).unwrap().activations, 1);
    assert_eq!(devices.get(node(2)).unwrap().activations, 1);
}

#[test]
fn repeated_lifecycle_events_are_forwarded_deterministically() {
    let mut active = true;
    let mut devices = manager_with_two();

    for _ in 0..2 {
        apply_session_event(SessionEvent::PauseSession, &mut active, &mut devices).unwrap();
    }
    for _ in 0..2 {
        apply_session_event(SessionEvent::ActivateSession, &mut active, &mut devices).unwrap();
    }

    assert!(active);
    for (_, device) in devices.iter() {
        assert_eq!(device.pauses, 2);
        assert_eq!(device.activations, 2);
    }
}

#[test]
fn empty_manager_preserves_session_transitions() {
    let mut active = true;
    let mut devices = DrmDeviceManager::<FakeDevice, u32>::new();

    apply_session_event(SessionEvent::PauseSession, &mut active, &mut devices).unwrap();
    assert!(!active);
    apply_session_event(SessionEvent::ActivateSession, &mut active, &mut devices).unwrap();
    assert!(active);
}
