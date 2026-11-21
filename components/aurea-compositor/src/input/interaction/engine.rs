// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Stateful coordinator for normalized pointing gestures.

use super::{GestureProfile, PointingGesture, PointingInteraction};

/// Resolves completed provider gestures using the active desktop profile.
///
/// Providers remain responsible for hardware event interpretation. This
/// engine never guesses unavailable hardware capabilities and does not own
/// Wayland protocol delivery.
#[derive(Debug, Clone, Default)]
pub struct InteractionEngine {
    profile: GestureProfile,
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

    /// Converts a normalized gesture into an optional desktop intent.
    pub fn resolve(&self, gesture: PointingGesture) -> Option<PointingInteraction> {
        self.profile.resolve(gesture)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::interaction::PointingInteraction;

    #[test]
    fn three_finger_swipe_resolves_workspace_direction() {
        let engine = InteractionEngine::new();
        assert_eq!(
            engine.resolve(PointingGesture::Swipe {
                fingers: 3,
                delta_x: -80.0,
                delta_y: 2.0,
            }),
            Some(PointingInteraction::NextWorkspace)
        );
    }

    #[test]
    fn pinch_resolves_overview() {
        let engine = InteractionEngine::new();
        assert_eq!(
            engine.resolve(PointingGesture::Pinch {
                fingers: 4,
                scale: 0.8,
                rotation: 0.0,
            }),
            Some(PointingInteraction::Overview)
        );
    }

    #[test]
    fn invalid_or_unrelated_swipe_is_ignored() {
        let engine = InteractionEngine::new();
        assert_eq!(
            engine.resolve(PointingGesture::Swipe {
                fingers: 3,
                delta_x: f64::NAN,
                delta_y: 0.0,
            }),
            None
        );
        assert_eq!(
            engine.resolve(PointingGesture::Swipe {
                fingers: 2,
                delta_x: 100.0,
                delta_y: 0.0,
            }),
            None
        );
    }
}
