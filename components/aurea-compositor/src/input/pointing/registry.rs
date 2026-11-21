// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Multi-device inventory and lifecycle notifications for pointing hardware.

use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
};

use aurea_desktop_api::input::{
    InputDeviceDescriptor, InputDeviceEvent, InputDeviceId, InputDeviceKind,
    InputDeviceProvider,
};

#[derive(Default)]
struct RegistryState {
    devices: HashMap<InputDeviceId, InputDeviceDescriptor>,
    subscribers: Vec<Sender<InputDeviceEvent>>,
}

/// Coordinates the discovered pointing devices attached to one compositor seat.
///
/// The service deliberately stores device capabilities instead of assuming
/// that every device in a category supports the same features. Multiple
/// devices of the same or different kinds can be registered simultaneously.
#[derive(Default)]
pub struct PointingDeviceService {
    state: Mutex<RegistryState>,
}

impl PointingDeviceService {
    /// Creates an empty pointing-device service.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds or refreshes a discovered pointing device.
    ///
    /// Keyboard-only and composite devices are not classified as pointing
    /// devices by this service; their lifecycle belongs to other input
    /// services.
    pub fn connect(&self, descriptor: InputDeviceDescriptor) -> bool {
        if !is_pointing_device(descriptor.kind) {
            return false;
        }

        let Ok(mut state) = self.state.lock() else {
            return false;
        };

        let event = if state.devices.contains_key(&descriptor.id) {
            InputDeviceEvent::CapabilitiesChanged(descriptor.clone())
        } else {
            InputDeviceEvent::Connected(descriptor.clone())
        };
        state.devices.insert(descriptor.id, descriptor);
        publish(&mut state, event);
        true
    }

    /// Removes a device and notifies subscribers when it was registered.
    pub fn disconnect(&self, id: InputDeviceId) -> bool {
        let Ok(mut state) = self.state.lock() else {
            return false;
        };

        if state.devices.remove(&id).is_none() {
            return false;
        }

        publish(&mut state, InputDeviceEvent::Disconnected(id));
        true
    }

    /// Reports whether at least one device of the requested kind is connected.
    pub fn has_kind(&self, kind: InputDeviceKind) -> bool {
        self.state
            .lock()
            .map(|state| state.devices.values().any(|device| device.kind == kind))
            .unwrap_or(false)
    }

    /// Returns the number of currently registered pointing devices.
    pub fn device_count(&self) -> usize {
        self.state
            .lock()
            .map(|state| state.devices.len())
            .unwrap_or_default()
    }
}

impl InputDeviceProvider for PointingDeviceService {
    fn devices(&self) -> Vec<InputDeviceDescriptor> {
        self.state
            .lock()
            .map(|state| state.devices.values().cloned().collect())
            .unwrap_or_default()
    }

    fn device(&self, id: InputDeviceId) -> Option<InputDeviceDescriptor> {
        self.state
            .lock()
            .ok()
            .and_then(|state| state.devices.get(&id).cloned())
    }

    fn subscribe(&self) -> Receiver<InputDeviceEvent> {
        let (sender, receiver) = mpsc::channel();
        if let Ok(mut state) = self.state.lock() {
            state.subscribers.push(sender);
        }
        receiver
    }
}

fn is_pointing_device(kind: InputDeviceKind) -> bool {
    matches!(
        kind,
        InputDeviceKind::Mouse
            | InputDeviceKind::Touchpad
            | InputDeviceKind::Tablet
            | InputDeviceKind::Touchscreen
    )
}

fn publish(state: &mut RegistryState, event: InputDeviceEvent) {
    state
        .subscribers
        .retain(|subscriber| subscriber.send(event.clone()).is_ok());
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurea_desktop_api::input::{InputDeviceCapabilities, StandardInputCapability};

    fn descriptor(id: u64, kind: InputDeviceKind) -> InputDeviceDescriptor {
        InputDeviceDescriptor {
            id: InputDeviceId(id),
            kind,
            name: format!("Test device {id}"),
            vendor_id: None,
            product_id: None,
            capabilities: InputDeviceCapabilities::new([
                StandardInputCapability::PointerMotion,
            ]),
        }
    }

    #[test]
    fn service_tracks_multiple_pointing_devices() {
        let service = PointingDeviceService::new();

        assert!(service.connect(descriptor(1, InputDeviceKind::Mouse)));
        assert!(service.connect(descriptor(2, InputDeviceKind::Touchpad)));
        assert!(service.connect(descriptor(3, InputDeviceKind::Tablet)));
        assert!(service.connect(descriptor(4, InputDeviceKind::Touchscreen)));
        assert_eq!(service.device_count(), 4);
    }

    #[test]
    fn service_rejects_non_pointing_devices() {
        let service = PointingDeviceService::new();

        assert!(!service.connect(descriptor(1, InputDeviceKind::Keyboard)));
        assert_eq!(service.device_count(), 0);
    }

    #[test]
    fn reconnecting_same_device_publishes_updated_capabilities() {
        let service = PointingDeviceService::new();
        let events = service.subscribe();
        let original = descriptor(8, InputDeviceKind::Mouse);
        let updated = InputDeviceDescriptor {
            capabilities: InputDeviceCapabilities::new([
                StandardInputCapability::PointerMotion,
                StandardInputCapability::PointerButtons,
                StandardInputCapability::PointerMiddleButton,
            ]),
            ..original.clone()
        };

        assert!(service.connect(original.clone()));
        assert_eq!(events.recv().unwrap(), InputDeviceEvent::Connected(original));
        assert!(service.connect(updated.clone()));
        assert_eq!(
            events.recv().unwrap(),
            InputDeviceEvent::CapabilitiesChanged(updated.clone())
        );
        assert_eq!(service.device(InputDeviceId(8)), Some(updated));
        assert_eq!(service.device_count(), 1);
    }

    #[test]
    fn disconnecting_unknown_device_does_not_change_registry() {
        let service = PointingDeviceService::new();
        assert!(!service.disconnect(InputDeviceId(404)));
        assert_eq!(service.device_count(), 0);
    }

    #[test]
    fn dropped_subscribers_are_pruned_on_next_event() {
        let service = PointingDeviceService::new();
        let receiver = service.subscribe();
        drop(receiver);

        assert!(service.connect(descriptor(9, InputDeviceKind::Mouse)));
        let state = service.state.lock().unwrap();
        assert!(state.subscribers.is_empty());
    }

    #[test]
    fn service_publishes_connection_and_disconnection_events() {
        let service = PointingDeviceService::new();
        let events = service.subscribe();
        let device = descriptor(7, InputDeviceKind::Touchpad);

        assert!(service.connect(device.clone()));
        assert_eq!(events.recv().unwrap(), InputDeviceEvent::Connected(device));
        assert!(service.disconnect(InputDeviceId(7)));
        assert_eq!(
            events.recv().unwrap(),
            InputDeviceEvent::Disconnected(InputDeviceId(7))
        );
    }
}
