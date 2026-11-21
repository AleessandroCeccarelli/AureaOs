// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Stateful coordinator for normalized pointing gestures.

use std::collections::VecDeque;

use super::{GestureProfile, PointingGesture, PointingInteraction};

#[derive(Debug, Clone, Copy)]
enum ActiveGesture {
    Swipe { fingers: u32, delta_x: f64, delta_y: f64 },
    Pinch { fingers: u32, scale: f64, rotation: f64 },
    Hold { fingers: u32 },
}

/// Resolves provider gestures and queues semantic desktop intents.
///
/// Providers interpret hardware events; this engine applies desktop policy.
/// Wayland delivery remains a separate compositor responsibility.
#[derive(Debug, Clone, Default)]
pub struct InteractionEngine {
    profile: GestureProfile,
    active: Option<ActiveGesture>,
    pending: VecDeque<PointingInteraction>,
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

    /// Starts a normalized swipe sequence.
    pub fn begin_swipe(&mut self, fingers: u32) {
        self.active = Some(ActiveGesture::Swipe {
            fingers,
            delta_x: 0.0,
            delta_y: 0.0,
        });
    }

    /// Adds a finite movement sample to the active swipe sequence.
    pub fn update_swipe(&mut self, delta_x: f64, delta_y: f64) {
        if !delta_x.is_finite() || !delta_y.is_finite() {
            return;
        }
        if let Some(ActiveGesture::Swipe {
            delta_x: total_x,
            delta_y: total_y,
            ..
        }) = self.active.as_mut()
        {
            *total_x += delta_x;
            *total_y += delta_y;
        }
    }

    /// Completes or discards the active swipe sequence.
    pub fn end_swipe(&mut self, cancelled: bool) {
        let Some(ActiveGesture::Swipe {
            fingers,
            delta_x,
            delta_y,
        }) = self.active.take()
        else {
            return;
        };
        if !cancelled {
            self.resolve_and_queue(PointingGesture::Swipe {
                fingers,
                delta_x,
                delta_y,
            });
        }
    }

    /// Starts a normalized pinch sequence.
    pub fn begin_pinch(&mut self, fingers: u32) {
        self.active = Some(ActiveGesture::Pinch {
            fingers,
            scale: 1.0,
            rotation: 0.0,
        });
    }

    /// Updates scale and rotation for the active pinch sequence.
    pub fn update_pinch(&mut self, scale: f64, rotation: f64) {
        if !scale.is_finite() || !rotation.is_finite() {
            return;
        }
        if let Some(ActiveGesture::Pinch {
            scale: total_scale,
            rotation: total_rotation,
            ..
        }) = self.active.as_mut()
        {
            *total_scale = scale;
            *total_rotation += rotation;
        }
    }

    /// Completes or discards the active pinch sequence.
    pub fn end_pinch(&mut self, cancelled: bool) {
        let Some(ActiveGesture::Pinch {
            fingers,
            scale,
            rotation,
        }) = self.active.take()
        else {
            return;
        };
        if !cancelled {
            self.resolve_and_queue(PointingGesture::Pinch {
                fingers,
                scale,
                rotation,
            });
        }
    }

    /// Starts a hold sequence after the provider's hold threshold is reached.
    pub fn begin_hold(&mut self, fingers: u32) {
        self.active = Some(ActiveGesture::Hold { fingers });
    }

    /// Completes or discards the active hold sequence.
    pub fn end_hold(&mut self, cancelled: bool) {
        let Some(ActiveGesture::Hold { fingers }) = self.active.take() else {
            return;
        };
        if !cancelled {
            self.resolve_and_queue(PointingGesture::Hold { fingers });
        }
    }

    /// Returns and removes the next pending desktop interaction.
    pub fn take_pending_interaction(&mut self) -> Option<PointingInteraction> {
        self.pending.pop_front()
    }

    /// Converts a standalone normalized gesture into a desktop intent.
    pub fn resolve(&self, gesture: PointingGesture) -> Option<PointingInteraction> {
        self.profile.resolve(gesture)
    }

    fn resolve_and_queue(&mut self, gesture: PointingGesture) {
        if let Some(interaction) = self.resolve(gesture) {
            self.pending.push_back(interaction);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completed_three_finger_swipe_queues_workspace_action() {
        let mut engine = InteractionEngine::new();
        engine.begin_swipe(3);
        engine.update_swipe(-80.0, 2.0);
        engine.end_swipe(false);

        assert_eq!(
            engine.take_pending_interaction(),
            Some(PointingInteraction::NextWorkspace)
        );
        assert_eq!(engine.take_pending_interaction(), None);
    }

    #[test]
    fn cancelled_swipe_does_not_queue_action() {
        let mut engine = InteractionEngine::new();
        engine.begin_swipe(3);
        engine.update_swipe(-100.0, 0.0);
        engine.end_swipe(true);

        assert_eq!(engine.take_pending_interaction(), None);
    }

    #[test]
    fn completed_pinch_queues_overview_action() {
        let mut engine = InteractionEngine::new();
        engine.begin_pinch(4);
        engine.update_pinch(0.8, 0.0);
        engine.end_pinch(false);

        assert_eq!(
            engine.take_pending_interaction(),
            Some(PointingInteraction::Overview)
        );
    }
}
