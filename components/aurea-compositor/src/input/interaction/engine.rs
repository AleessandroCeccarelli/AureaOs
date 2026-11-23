// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Per-device coordinator for normalized pointing gestures.

use std::collections::{HashMap, VecDeque};

use aurea_desktop_api::input::InputDeviceId;

use super::{GestureProfile, PointingGesture, PointingInteraction};

/// Maximum number of completed interactions waiting for the compositor.
const MAX_PENDING_INTERACTIONS: usize = 64;

#[derive(Debug, Clone, Copy)]
enum ActiveGesture {
    Swipe {
        fingers: u32,
        delta_x: f64,
        delta_y: f64,
    },
    Pinch {
        fingers: u32,
        scale: f64,
        rotation: f64,
    },
    Hold {
        fingers: u32,
    },
}

/// Resolves provider gestures and queues semantic desktop intents.
///
/// Active sequences are isolated by stable device identity, so events from
/// multiple touchpads cannot overwrite one another.
#[derive(Debug, Clone, Default)]
pub struct InteractionEngine {
    profile: GestureProfile,
    active: HashMap<InputDeviceId, ActiveGesture>,
    pending: VecDeque<(InputDeviceId, PointingInteraction)>,
}

impl InteractionEngine {
    /// Creates an interaction engine with AureaOS default bindings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces the active gesture policy.
    pub fn set_profile(&mut self, profile: GestureProfile) {
        self.profile = profile;
    }

    /// Returns the current gesture policy.
    pub fn profile(&self) -> &GestureProfile {
        &self.profile
    }

    /// Starts a swipe sequence for one physical device.
    pub fn begin_swipe(&mut self, device: InputDeviceId, fingers: u32) {
        self.active.insert(
            device,
            ActiveGesture::Swipe {
                fingers,
                delta_x: 0.0,
                delta_y: 0.0,
            },
        );
    }

    /// Adds a finite movement sample to that device's active swipe.
    pub fn update_swipe(&mut self, device: InputDeviceId, delta_x: f64, delta_y: f64) {
        if !delta_x.is_finite() || !delta_y.is_finite() {
            return;
        }
        if let Some(ActiveGesture::Swipe {
            delta_x: total_x,
            delta_y: total_y,
            ..
        }) = self.active.get_mut(&device)
        {
            *total_x += delta_x;
            *total_y += delta_y;
        }
    }

    /// Completes or discards one device's swipe sequence.
    pub fn end_swipe(&mut self, device: InputDeviceId, cancelled: bool) {
        let Some(ActiveGesture::Swipe {
            fingers,
            delta_x,
            delta_y,
        }) = self.active.get(&device).copied()
        else {
            return;
        };
        self.active.remove(&device);
        if !cancelled {
            self.resolve_and_queue(
                device,
                PointingGesture::Swipe {
                    fingers,
                    delta_x,
                    delta_y,
                },
            );
        }
    }

    /// Starts a pinch sequence for one physical device.
    pub fn begin_pinch(&mut self, device: InputDeviceId, fingers: u32) {
        self.active.insert(
            device,
            ActiveGesture::Pinch {
                fingers,
                scale: 1.0,
                rotation: 0.0,
            },
        );
    }

    /// Updates scale and rotation for that device's active pinch.
    pub fn update_pinch(&mut self, device: InputDeviceId, scale: f64, rotation: f64) {
        if !scale.is_finite() || !rotation.is_finite() {
            return;
        }
        if let Some(ActiveGesture::Pinch {
            scale: total_scale,
            rotation: total_rotation,
            ..
        }) = self.active.get_mut(&device)
        {
            *total_scale = scale;
            *total_rotation += rotation;
        }
    }

    /// Completes or discards one device's pinch sequence.
    pub fn end_pinch(&mut self, device: InputDeviceId, cancelled: bool) {
        let Some(ActiveGesture::Pinch {
            fingers,
            scale,
            rotation,
        }) = self.active.get(&device).copied()
        else {
            return;
        };
        self.active.remove(&device);
        if !cancelled {
            self.resolve_and_queue(
                device,
                PointingGesture::Pinch {
                    fingers,
                    scale,
                    rotation,
                },
            );
        }
    }

    /// Starts a hold sequence for one physical device.
    pub fn begin_hold(&mut self, device: InputDeviceId, fingers: u32) {
        self.active.insert(device, ActiveGesture::Hold { fingers });
    }

    /// Completes or discards one device's hold sequence.
    pub fn end_hold(&mut self, device: InputDeviceId, cancelled: bool) {
        let Some(ActiveGesture::Hold { fingers }) = self.active.get(&device).copied() else {
            return;
        };
        self.active.remove(&device);
        if !cancelled {
            self.resolve_and_queue(device, PointingGesture::Hold { fingers });
        }
    }

    /// Clears all active sequences when a device disconnects.
    pub fn remove_device(&mut self, device: InputDeviceId) {
        self.active.remove(&device);
        self.pending.retain(|(source, _)| *source != device);
    }

    /// Returns and removes the next pending desktop interaction.
    pub fn take_pending_interaction(&mut self) -> Option<PointingInteraction> {
        self.pending.pop_front().map(|(_, interaction)| interaction)
    }

    /// Converts a standalone normalized gesture into a desktop intent.
    pub fn resolve(&self, gesture: PointingGesture) -> Option<PointingInteraction> {
        self.profile.resolve(gesture)
    }

    fn resolve_and_queue(&mut self, device: InputDeviceId, gesture: PointingGesture) {
        if let Some(interaction) = self.resolve(gesture) {
            if self.pending.len() == MAX_PENDING_INTERACTIONS {
                self.pending.pop_front();
            }
            self.pending.push_back((device, interaction));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completed_three_finger_swipe_queues_workspace_action() {
        let mut engine = InteractionEngine::new();
        let device = InputDeviceId(1);
        engine.begin_swipe(device, 3);
        engine.update_swipe(device, -80.0, 2.0);
        engine.end_swipe(device, false);

        assert_eq!(
            engine.take_pending_interaction(),
            Some(PointingInteraction::NextWorkspace)
        );
        assert_eq!(engine.take_pending_interaction(), None);
    }

    #[test]
    fn simultaneous_devices_keep_independent_swipe_sequences() {
        let mut engine = InteractionEngine::new();
        let first = InputDeviceId(1);
        let second = InputDeviceId(2);
        engine.begin_swipe(first, 3);
        engine.begin_swipe(second, 3);
        engine.update_swipe(first, -80.0, 0.0);
        engine.update_swipe(second, 80.0, 0.0);
        engine.end_swipe(first, false);
        engine.end_swipe(second, false);

        assert_eq!(
            engine.take_pending_interaction(),
            Some(PointingInteraction::NextWorkspace)
        );
        assert_eq!(
            engine.take_pending_interaction(),
            Some(PointingInteraction::PreviousWorkspace)
        );
    }

    #[test]
    fn cancelled_swipe_does_not_queue_action() {
        let mut engine = InteractionEngine::new();
        let device = InputDeviceId(3);
        engine.begin_swipe(device, 3);
        engine.update_swipe(device, -100.0, 0.0);
        engine.end_swipe(device, true);

        assert_eq!(engine.take_pending_interaction(), None);
    }

    #[test]
    fn ending_a_different_gesture_kind_does_not_discard_active_sequence() {
        let mut engine = InteractionEngine::new();
        let device = InputDeviceId(5);
        engine.begin_swipe(device, 3);
        engine.update_swipe(device, -80.0, 0.0);

        engine.end_pinch(device, false);
        engine.end_swipe(device, false);

        assert_eq!(
            engine.take_pending_interaction(),
            Some(PointingInteraction::NextWorkspace)
        );
    }

    #[test]
    fn disconnect_removes_completed_but_not_yet_dispatched_actions() {
        let mut engine = InteractionEngine::new();
        let device = InputDeviceId(6);
        engine.begin_swipe(device, 3);
        engine.update_swipe(device, -80.0, 0.0);
        engine.end_swipe(device, false);

        engine.remove_device(device);

        assert_eq!(engine.take_pending_interaction(), None);
    }

    #[test]
    fn disconnecting_one_device_preserves_other_device_actions() {
        let mut engine = InteractionEngine::new();
        let removed = InputDeviceId(7);
        let retained = InputDeviceId(8);

        engine.begin_swipe(removed, 3);
        engine.update_swipe(removed, -80.0, 0.0);
        engine.end_swipe(removed, false);
        engine.begin_swipe(retained, 3);
        engine.update_swipe(retained, 80.0, 0.0);
        engine.end_swipe(retained, false);

        engine.remove_device(removed);

        assert_eq!(
            engine.take_pending_interaction(),
            Some(PointingInteraction::PreviousWorkspace)
        );
        assert_eq!(engine.take_pending_interaction(), None);
    }

    #[test]
    fn pending_interaction_queue_is_bounded() {
        let mut engine = InteractionEngine::new();
        let device = InputDeviceId(9);

        for _ in 0..(MAX_PENDING_INTERACTIONS + 1) {
            engine.begin_swipe(device, 3);
            engine.update_swipe(device, -80.0, 0.0);
            engine.end_swipe(device, false);
        }

        let mut received = 0;
        while engine.take_pending_interaction().is_some() {
            received += 1;
        }

        assert_eq!(received, MAX_PENDING_INTERACTIONS);
    }

    #[test]
    fn completed_pinch_queues_overview_action() {
        let mut engine = InteractionEngine::new();
        let device = InputDeviceId(4);
        engine.begin_pinch(device, 4);
        engine.update_pinch(device, 0.8, 0.0);
        engine.end_pinch(device, false);

        assert_eq!(
            engine.take_pending_interaction(),
            Some(PointingInteraction::Overview)
        );
    }
}
