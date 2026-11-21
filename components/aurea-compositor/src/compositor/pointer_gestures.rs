// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Forward normalized libinput gestures to Wayland pointer-gestures clients.

use smithay::{
    input::pointer::{
        GestureHoldBeginEvent, GestureHoldEndEvent, GesturePinchBeginEvent,
        GesturePinchEndEvent, GesturePinchUpdateEvent, GestureSwipeBeginEvent,
        GestureSwipeEndEvent, GestureSwipeUpdateEvent,
    },
    utils::SERIAL_COUNTER,
};

use super::AureaCompositor;

impl AureaCompositor {
    /// Retrieves the next semantic pointing interaction for the desktop shell.
    pub fn take_pending_pointing_interaction(
        &mut self,
    ) -> Option<crate::input::interaction::PointingInteraction> {
        self.interaction_engine.take_pending_interaction()
    }

    /// Sends a touchpad swipe-begin event to the focused Wayland client.
    pub fn handle_swipe_begin(&mut self, fingers: u32, time: smithay::backend::input::InputTime) {
        self.interaction_engine.begin_swipe(fingers);
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
        delta_x: f64,
        delta_y: f64,
        time: u32,
    ) {
        if !delta_x.is_finite() || !delta_y.is_finite() {
            return;
        }
        self.interaction_engine.update_swipe(delta_x, delta_y);
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
        cancelled: bool,
        time: smithay::backend::input::InputTime,
    ) {
        self.interaction_engine.end_swipe(cancelled);
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
    pub fn handle_pinch_begin(&mut self, fingers: u32, time: smithay::backend::input::InputTime) {
        self.interaction_engine.begin_pinch(fingers);
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
        self.interaction_engine.update_pinch(scale, rotation);
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
        cancelled: bool,
        time: smithay::backend::input::InputTime,
    ) {
        self.interaction_engine.end_pinch(cancelled);
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
    pub fn handle_hold_begin(&mut self, fingers: u32, time: smithay::backend::input::InputTime) {
        self.interaction_engine.hold(fingers);
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
        cancelled: bool,
        time: smithay::backend::input::InputTime,
    ) {
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
