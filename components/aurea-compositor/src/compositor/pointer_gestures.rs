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

use aurea_desktop_api::desktop_interaction::{DesktopInteraction, DesktopPosition};

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
            Some(_) => false,
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

    fn take(&mut self) -> Option<(aurea_desktop_api::input::InputDeviceId, PointerGestureKind)> {
        self.0.take()
    }

    fn take_device(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
    ) -> Option<PointerGestureKind> {
        if self.0.is_some_and(|(owner, _)| owner == device) {
            self.0.take().map(|(_, kind)| kind)
        } else {
            None
        }
    }
}

impl AureaCompositor {
    /// Cancels the active Wayland gesture stream when the native session pauses.
    ///
    /// A VT switch can interrupt libinput without delivering the matching end
    /// event. Clear both the semantic gesture accumulator and protocol stream.
    #[cfg(feature = "drm")]
    pub(crate) fn cancel_active_pointer_gesture(&mut self, time: u32) {
        if let Some((device, kind)) = self.pointer_gesture_owner.take() {
            self.cancel_pointer_gesture(device, kind, time);
        }
    }

    /// Cancels an unfinished gesture before its physical device is unregistered.
    pub(crate) fn clear_pointing_device_state(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        time: u32,
    ) {
        if let Some(kind) = self.pointer_gesture_owner.take_device(device) {
            self.cancel_pointer_gesture(device, kind, time);
        }
        self.remove_pointing_device_state(device);
    }

    fn cancel_pointer_gesture(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        kind: PointerGestureKind,
        time: u32,
    ) {
        match kind {
            PointerGestureKind::Swipe => self.interaction_engine.end_swipe(device, true),
            PointerGestureKind::Pinch => self.interaction_engine.end_pinch(device, true),
            PointerGestureKind::Hold => self.interaction_engine.end_hold(device, true),
        }

        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        let serial = SERIAL_COUNTER.next_serial();
        match kind {
            PointerGestureKind::Swipe => pointer.gesture_swipe_end(
                self,
                &GestureSwipeEndEvent { serial, time, cancelled: true },
            ),
            PointerGestureKind::Pinch => pointer.gesture_pinch_end(
                self,
                &GesturePinchEndEvent { serial, time, cancelled: true },
            ),
            PointerGestureKind::Hold => pointer.gesture_hold_end(
                self,
                &GestureHoldEndEvent { serial, time, cancelled: true },
            ),
        }
        pointer.frame(self);
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
                crate::input::interaction::PointingInteraction::AppExpose => {
                    Some(DesktopInteraction::ShowAppExpose)
                }
                crate::input::interaction::PointingInteraction::ContextMenu => {
                    self.seat.get_pointer().map(|pointer| {
                        let location = pointer.current_location();
                        DesktopInteraction::ShowContextMenu {
                            position: DesktopPosition {
                                x: location.x.round() as i32,
                                y: location.y.round() as i32,
                            },
                        }
                    })
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
        time: u32,
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
        time: u32,
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
        time: u32,
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
        time: u32,
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
        time: u32,
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
        time: u32,
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
        time: u32,
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
        assert_eq!(
            owner.take_device(removed),
            Some(PointerGestureKind::Hold)
        );

        assert!(owner.begin(retained, PointerGestureKind::Pinch));
        assert!(owner.owns(retained, PointerGestureKind::Pinch));
    }

    #[test]
    fn duplicate_begin_for_same_sequence_is_rejected() {
        let mut owner = PointerGestureOwner::default();
        let device = InputDeviceId(7);
        assert!(owner.begin(device, PointerGestureKind::Swipe));
        assert!(!owner.begin(device, PointerGestureKind::Swipe));
        assert!(!owner.begin(device, PointerGestureKind::Pinch));
        assert_eq!(owner.take(), Some((device, PointerGestureKind::Swipe)));
    }

    #[test]
    fn device_removal_takes_only_its_active_gesture_owner() {
        let mut owner = PointerGestureOwner::default();
        let device = InputDeviceId(10);
        assert!(owner.begin(device, PointerGestureKind::Hold));
        assert_eq!(owner.take_device(InputDeviceId(11)), None);
        assert_eq!(owner.take_device(device), Some(PointerGestureKind::Hold));
        assert_eq!(owner.take(), None);
    }

    #[test]
    fn taking_active_owner_clears_the_stream_for_session_cancellation() {
        let mut owner = PointerGestureOwner::default();
        let device = InputDeviceId(8);

        assert!(owner.begin(device, PointerGestureKind::Pinch));
        assert_eq!(owner.take(), Some((device, PointerGestureKind::Pinch)));
        assert_eq!(owner.take(), None);
        assert!(owner.begin(InputDeviceId(9), PointerGestureKind::Swipe));
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

    #[test]
    fn completed_three_finger_swipe_reaches_shell_as_workspace_change() {
        use aurea_desktop_api::desktop_interaction::{
            DesktopInteraction, desktop_interaction_channel,
        };
        use smithay::reexports::wayland_server::Display;

        let display = Display::<super::AureaCompositor>::new()
            .expect("Wayland display should initialize for the gesture integration test");
        let (sender, receiver) = desktop_interaction_channel();
        let mut compositor = super::AureaCompositor::new(&display.handle(), sender);
        let device = InputDeviceId(42);

        compositor.handle_swipe_begin(device, 3, 10);
        compositor.handle_swipe_update(device, -80.0, 0.0, 11);
        compositor.handle_swipe_end(device, false, 12);

        assert_eq!(compositor.active_workspace(), 1);
        assert_eq!(
            receiver.try_recv(),
            Ok(DesktopInteraction::WorkspaceChanged { index: 1 })
        );
    }

    #[test]
    fn cancelled_three_finger_swipe_does_not_reach_shell() {
        use aurea_desktop_api::desktop_interaction::desktop_interaction_channel;
        use smithay::reexports::wayland_server::Display;
        use std::sync::mpsc::TryRecvError;

        let display = Display::<super::AureaCompositor>::new()
            .expect("Wayland display should initialize for the cancelled gesture test");
        let (sender, receiver) = desktop_interaction_channel();
        let mut compositor = super::AureaCompositor::new(&display.handle(), sender);
        let device = InputDeviceId(43);

        compositor.handle_swipe_begin(device, 3, 20);
        compositor.handle_swipe_update(device, -100.0, 0.0, 21);
        compositor.handle_swipe_end(device, true, 22);

        assert_eq!(compositor.active_workspace(), 0);
        assert_eq!(receiver.try_recv(), Err(TryRecvError::Empty));
    }
}
