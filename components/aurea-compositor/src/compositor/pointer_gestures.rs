// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Forward normalized libinput gestures to Wayland pointer-gestures clients.

use smithay::{
    input::pointer::{
        GestureHoldBeginEvent, GestureHoldEndEvent, GesturePinchBeginEvent, GesturePinchEndEvent,
        GesturePinchUpdateEvent, GestureSwipeBeginEvent, GestureSwipeEndEvent,
        GestureSwipeUpdateEvent,
    },
    utils::SERIAL_COUNTER,
};

use aurea_desktop_api::desktop_interaction::DesktopInteraction;

use super::AureaCompositor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PointerGestureKind {
    Swipe,
    Pinch,
    Hold,
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct PointerGestureOwner(
    Option<(aurea_desktop_api::input::InputDeviceId, PointerGestureKind)>,
);

impl PointerGestureOwner {
    fn begin(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        kind: PointerGestureKind,
    ) -> bool {
        match self.0 {
            None => {
                self.0 = Some((device, kind));
                true
            }
            Some(owner) => owner == (device, kind),
        }
    }

    fn owns(
        &self,
        device: aurea_desktop_api::input::InputDeviceId,
        kind: PointerGestureKind,
    ) -> bool {
        self.0 == Some((device, kind))
    }

    fn end(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        kind: PointerGestureKind,
    ) -> bool {
        if !self.owns(device, kind) {
            return false;
        }
        self.0 = None;
        true
    }

    fn remove_device(&mut self, device: aurea_desktop_api::input::InputDeviceId) {
        if self.0.is_some_and(|(owner, _)| owner == device) {
            self.0 = None;
        }
    }
}

impl AureaCompositor {
    /// Drops any unfinished gesture sequence owned by a disconnected device.
    pub fn clear_pointing_device_state(&mut self, device: aurea_desktop_api::input::InputDeviceId) {
        self.pointer_gesture_owner.remove_device(device);
        self.remove_pointing_device_state(device);
    }

    fn publish_completed_interactions(&mut self) {
        while let Some(interaction) = self.interaction_engine.take_pending_interaction() {
            let desktop_interaction = match interaction {
                crate::input::interaction::PointingInteraction::PreviousWorkspace => {
                    let current = self.active_workspace();
                    let previous =
                        (current + crate::window_manager::WindowManager::WORKSPACE_COUNT - 1)
                            % crate::window_manager::WindowManager::WORKSPACE_COUNT;
                    self.switch_workspace(previous);
                    None
                }
                crate::input::interaction::PointingInteraction::NextWorkspace => {
                    let current = self.active_workspace();
                    let next =
                        (current + 1) % crate::window_manager::WindowManager::WORKSPACE_COUNT;
                    self.switch_workspace(next);
                    None
                }
                crate::input::interaction::PointingInteraction::Overview => {
                    Some(DesktopInteraction::ShowOverview)
                }
                crate::input::interaction::PointingInteraction::ContextMenu => {
                    Some(DesktopInteraction::ShowContextMenu)
                }
            };
            if let Some(desktop_interaction) = desktop_interaction {
                if self.desktop_interactions.send(desktop_interaction).is_err() {
                    tracing::debug!("Shell interaction receiver is no longer available");
                    break;
                }
            }
        }
    }

    /// Sends a touchpad swipe-begin event to the focused Wayland client.
    pub fn handle_swipe_begin(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        fingers: u32,
        time: smithay::backend::input::InputTime,
    ) {
        if !self
            .pointer_gesture_owner
            .begin(device, PointerGestureKind::Swipe)
        {
            return;
        }
        self.interaction_engine.begin_swipe(device, fingers);
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        pointer.gesture_swipe_begin(
            self,
            &GestureSwipeBeginEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time,
                fingers,
            },
        );
        pointer.frame(self);
    }

    /// Sends a touchpad swipe update to the focused Wayland client.
    pub fn handle_swipe_update(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        delta_x: f64,
        delta_y: f64,
        time: u32,
    ) {
        if !delta_x.is_finite() || !delta_y.is_finite() {
            return;
        }
        if !self
            .pointer_gesture_owner
            .owns(device, PointerGestureKind::Swipe)
        {
            return;
        }
        self.interaction_engine
            .update_swipe(device, delta_x, delta_y);
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        pointer.gesture_swipe_update(
            self,
            &GestureSwipeUpdateEvent {
                time,
                delta: (delta_x, delta_y).into(),
            },
        );
        pointer.frame(self);
    }

    /// Sends a touchpad swipe-end event to the focused Wayland client.
    pub fn handle_swipe_end(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        cancelled: bool,
        time: smithay::backend::input::InputTime,
    ) {
        if !self
            .pointer_gesture_owner
            .end(device, PointerGestureKind::Swipe)
        {
            return;
        }
        self.interaction_engine.end_swipe(device, cancelled);
        self.publish_completed_interactions();
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        pointer.gesture_swipe_end(
            self,
            &GestureSwipeEndEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time,
                cancelled,
            },
        );
        pointer.frame(self);
    }

    /// Sends a touchpad pinch-begin event to the focused Wayland client.
    pub fn handle_pinch_begin(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        fingers: u32,
        time: smithay::backend::input::InputTime,
    ) {
        if !self
            .pointer_gesture_owner
            .begin(device, PointerGestureKind::Pinch)
        {
            return;
        }
        self.interaction_engine.begin_pinch(device, fingers);
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        pointer.gesture_pinch_begin(
            self,
            &GesturePinchBeginEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time,
                fingers,
            },
        );
        pointer.frame(self);
    }

    /// Sends a touchpad pinch update to the focused Wayland client.
    pub fn handle_pinch_update(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        delta_x: f64,
        delta_y: f64,
        scale: f64,
        rotation: f64,
        time: smithay::backend::input::InputTime,
    ) {
        if !delta_x.is_finite()
            || !delta_y.is_finite()
            || !scale.is_finite()
            || !rotation.is_finite()
        {
            return;
        }
        if !self
            .pointer_gesture_owner
            .owns(device, PointerGestureKind::Pinch)
        {
            return;
        }
        self.interaction_engine
            .update_pinch(device, scale, rotation);
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        pointer.gesture_pinch_update(
            self,
            &GesturePinchUpdateEvent {
                time,
                delta: (delta_x, delta_y).into(),
                scale,
                rotation,
            },
        );
        pointer.frame(self);
    }

    /// Sends a touchpad pinch-end event to the focused Wayland client.
    pub fn handle_pinch_end(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        cancelled: bool,
        time: smithay::backend::input::InputTime,
    ) {
        if !self
            .pointer_gesture_owner
            .end(device, PointerGestureKind::Pinch)
        {
            return;
        }
        self.interaction_engine.end_pinch(device, cancelled);
        self.publish_completed_interactions();
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        pointer.gesture_pinch_end(
            self,
            &GesturePinchEndEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time,
                cancelled,
            },
        );
        pointer.frame(self);
    }

    /// Sends a touchpad hold-begin event to the focused Wayland client.
    pub fn handle_hold_begin(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        fingers: u32,
        time: smithay::backend::input::InputTime,
    ) {
        if !self
            .pointer_gesture_owner
            .begin(device, PointerGestureKind::Hold)
        {
            return;
        }
        self.interaction_engine.begin_hold(device, fingers);
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        pointer.gesture_hold_begin(
            self,
            &GestureHoldBeginEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time,
                fingers,
            },
        );
        pointer.frame(self);
    }

    /// Sends a touchpad hold-end event to the focused Wayland client.
    pub fn handle_hold_end(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        cancelled: bool,
        time: smithay::backend::input::InputTime,
    ) {
        if !self
            .pointer_gesture_owner
            .end(device, PointerGestureKind::Hold)
        {
            return;
        }
        self.interaction_engine.end_hold(device, cancelled);
        self.publish_completed_interactions();
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        pointer.gesture_hold_end(
            self,
            &GestureHoldEndEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time,
                cancelled,
            },
        );
        pointer.frame(self);
    }
}

#[cfg(test)]
mod owner_tests {
    use super::{PointerGestureKind, PointerGestureOwner};
    use aurea_desktop_api::input::InputDeviceId;

    #[test]
    fn only_one_device_owns_the_wayland_gesture_stream() {
        let mut owner = PointerGestureOwner::default();
        let first = InputDeviceId(1);
        let second = InputDeviceId(2);

        assert!(owner.begin(first, PointerGestureKind::Swipe));
        assert!(!owner.begin(second, PointerGestureKind::Swipe));
        assert!(!owner.owns(second, PointerGestureKind::Swipe));
        assert!(owner.owns(first, PointerGestureKind::Swipe));
    }

    #[test]
    fn different_gesture_kind_cannot_interrupt_active_sequence() {
        let mut owner = PointerGestureOwner::default();
        let device = InputDeviceId(3);

        assert!(owner.begin(device, PointerGestureKind::Swipe));
        assert!(!owner.begin(device, PointerGestureKind::Pinch));
        assert!(!owner.end(device, PointerGestureKind::Pinch));
        assert!(owner.owns(device, PointerGestureKind::Swipe));
    }

    #[test]
    fn disconnecting_owner_releases_stream_for_another_device() {
        let mut owner = PointerGestureOwner::default();
        let removed = InputDeviceId(4);
        let retained = InputDeviceId(5);

        assert!(owner.begin(removed, PointerGestureKind::Hold));
        owner.remove_device(removed);

        assert!(owner.begin(retained, PointerGestureKind::Pinch));
        assert!(owner.owns(retained, PointerGestureKind::Pinch));
    }

    #[test]
    fn ending_non_owner_does_not_release_active_sequence() {
        let mut owner = PointerGestureOwner::default();
        let first = InputDeviceId(6);
        let second = InputDeviceId(7);

        assert!(owner.begin(first, PointerGestureKind::Swipe));
        assert!(!owner.end(second, PointerGestureKind::Swipe));
        assert!(owner.owns(first, PointerGestureKind::Swipe));
        assert!(owner.end(first, PointerGestureKind::Swipe));
        assert!(!owner.owns(first, PointerGestureKind::Swipe));
    }
}
