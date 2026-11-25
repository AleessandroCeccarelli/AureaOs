// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Multi-device inventory and lifecycle notifications for pointing hardware.

use std::{
    collections::HashMap,
    sync::{
        Mutex,
        mpsc::{self, Receiver, Sender},
    },
};

use aurea_desktop_api::input::{
    InputDeviceDescriptor, InputDeviceEvent, InputDeviceId, InputDeviceKind, InputDeviceProvider,
    TouchpadApplyReport,
};

#[derive(Default)]
struct RegistryState {
    devices: HashMap<InputDeviceId, InputDeviceDescriptor>,
    touchpad_reports: HashMap<InputDeviceId, TouchpadApplyReport>,
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
    pub(crate) fn connect(&self, descriptor: InputDeviceDescriptor) -> bool {
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
        // A device can be re-announced with changed capabilities or a changed
        // classification. Never expose a stale touchpad apply report as if it
        // described a device that is no longer a touchpad.
        if descriptor.kind != InputDeviceKind::Touchpad {
            state.touchpad_reports.remove(&descriptor.id);
        }
        state.devices.insert(descriptor.id, descriptor);
        publish(&mut state, event);
        true
    }

    /// Removes a device and notifies subscribers when it was registered.
    pub(crate) fn disconnect(&self, id: InputDeviceId) -> bool {
        let Ok(mut state) = self.state.lock() else {
            return false;
        };

        state.touchpad_reports.remove(&id);
        if state.devices.remove(&id).is_none() {
            return false;
        }

        publish(&mut state, InputDeviceEvent::Disconnected(id));
        true
    }

    /// Stores the latest configuration outcome for one registered touchpad.
    pub fn set_touchpad_report(&self, id: InputDeviceId, report: TouchpadApplyReport) -> bool {
        let Ok(mut state) = self.state.lock() else {
            return false;
        };
        if !state
            .devices
            .get(&id)
            .is_some_and(|device| device.kind == InputDeviceKind::Touchpad)
        {
            return false;
        }
        state.touchpad_reports.insert(id, report);
        publish(
            &mut state,
            InputDeviceEvent::TouchpadConfigurationChanged(id, report),
        );
        true
    }

    /// Returns the latest configuration outcome for one registered touchpad.
    pub fn touchpad_report(&self, id: InputDeviceId) -> Option<TouchpadApplyReport> {
        self.state.lock().ok()?.touchpad_reports.get(&id).copied()
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

    fn touchpad_report(&self, id: InputDeviceId) -> Option<TouchpadApplyReport> {
        PointingDeviceService::touchpad_report(self, id)
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
            | InputDeviceKind::TabletPad
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
            persistent_id: None,
            capabilities: InputDeviceCapabilities::new([StandardInputCapability::PointerMotion]),
        }
    }

    #[test]
    fn service_tracks_multiple_pointing_devices() {
        let service = PointingDeviceService::new();

        assert!(service.connect(descriptor(1, InputDeviceKind::Mouse)));
        assert!(service.connect(descriptor(2, InputDeviceKind::Touchpad)));
        assert!(service.connect(descriptor(3, InputDeviceKind::Tablet)));
        assert!(service.connect(descriptor(4, InputDeviceKind::Touchscreen)));
        assert!(service.connect(descriptor(5, InputDeviceKind::TabletPad)));
        assert_eq!(service.device_count(), 5);
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
        assert_eq!(
            events.recv().unwrap(),
            InputDeviceEvent::Connected(original)
        );
        assert!(service.connect(updated.clone()));
        assert_eq!(
            events.recv().unwrap(),
            InputDeviceEvent::CapabilitiesChanged(updated.clone())
        );
        assert_eq!(service.device(InputDeviceId(8)), Some(updated));
        assert_eq!(service.device_count(), 1);
    }

    #[test]
    fn reclassification_away_from_touchpad_clears_old_touchpad_report() {
        let service = PointingDeviceService::new();
        let id = InputDeviceId(24);
        let report = TouchpadApplyReport {
            tap_to_click: aurea_desktop_api::input::TouchpadSettingStatus::Applied,
            tap_and_drag: aurea_desktop_api::input::TouchpadSettingStatus::Applied,
            tap_drag_lock: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            tap_button_map: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            two_finger_scroll: aurea_desktop_api::input::TouchpadSettingStatus::Applied,
            edge_scrolling: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            natural_scrolling: aurea_desktop_api::input::TouchpadSettingStatus::Applied,
            disable_while_typing: aurea_desktop_api::input::TouchpadSettingStatus::Applied,
            click_method: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            left_handed: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            middle_button_emulation: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            pointer_speed: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            acceleration_profile: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            send_events_mode: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
        };

        assert!(service.connect(descriptor(24, InputDeviceKind::Touchpad)));
        assert!(service.set_touchpad_report(id, report));
        assert!(service.connect(descriptor(24, InputDeviceKind::Mouse)));
        assert_eq!(service.device(id).unwrap().kind, InputDeviceKind::Mouse);
        assert_eq!(service.touchpad_report(id), None);
    }

    #[test]
    fn touchpad_report_is_available_only_for_registered_touchpads() {
        let service = PointingDeviceService::new();
        let id = InputDeviceId(21);
        let report = TouchpadApplyReport {
            tap_to_click: aurea_desktop_api::input::TouchpadSettingStatus::Applied,
            tap_and_drag: aurea_desktop_api::input::TouchpadSettingStatus::Applied,
            tap_drag_lock: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            tap_button_map: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            two_finger_scroll: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            edge_scrolling: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            natural_scrolling: aurea_desktop_api::input::TouchpadSettingStatus::Applied,
            disable_while_typing: aurea_desktop_api::input::TouchpadSettingStatus::Applied,
            click_method: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            left_handed: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            middle_button_emulation: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            pointer_speed: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            acceleration_profile: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            send_events_mode: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
        };

        assert!(!service.set_touchpad_report(id, report));
        assert!(service.connect(descriptor(21, InputDeviceKind::Touchpad)));
        assert!(service.set_touchpad_report(id, report));
        assert_eq!(service.touchpad_report(id), Some(report));
    }

    #[test]
    fn disconnect_removes_touchpad_configuration_report() {
        let service = PointingDeviceService::new();
        let id = InputDeviceId(22);
        let report = TouchpadApplyReport {
            tap_to_click: aurea_desktop_api::input::TouchpadSettingStatus::Applied,
            tap_and_drag: aurea_desktop_api::input::TouchpadSettingStatus::Applied,
            tap_drag_lock: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            tap_button_map: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            two_finger_scroll: aurea_desktop_api::input::TouchpadSettingStatus::Applied,
            edge_scrolling: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            natural_scrolling: aurea_desktop_api::input::TouchpadSettingStatus::Applied,
            disable_while_typing: aurea_desktop_api::input::TouchpadSettingStatus::Applied,
            click_method: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            left_handed: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            middle_button_emulation: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            pointer_speed: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            acceleration_profile: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            send_events_mode: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
        };

        service.connect(descriptor(22, InputDeviceKind::Touchpad));
        assert!(service.set_touchpad_report(id, report));
        assert!(service.disconnect(id));
        assert_eq!(service.touchpad_report(id), None);
    }

    #[test]
    fn mouse_cannot_receive_touchpad_configuration_report() {
        let service = PointingDeviceService::new();
        let id = InputDeviceId(23);
        let report = TouchpadApplyReport {
            tap_to_click: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            tap_and_drag: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            tap_drag_lock: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            tap_button_map: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            two_finger_scroll: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            edge_scrolling: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            natural_scrolling: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            disable_while_typing: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            disable_while_trackpointing: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            click_method: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            left_handed: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            middle_button_emulation: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            pointer_speed: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            acceleration_profile: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
            send_events_mode: aurea_desktop_api::input::TouchpadSettingStatus::Unsupported,
        };

        service.connect(descriptor(23, InputDeviceKind::Mouse));
        assert!(!service.set_touchpad_report(id, report));
        assert_eq!(service.touchpad_report(id), None);
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
