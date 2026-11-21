// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Default gesture bindings and validation boundaries.

use super::{PointingGesture, PointingInteraction};

/// A configurable gesture-to-desktop-action binding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GestureBinding {
    pub fingers: u32,
    pub interaction: PointingInteraction,
}

/// AureaOS default desktop gesture policy.
#[derive(Debug, Clone, PartialEq)]
pub struct GestureProfile {
    pub workspace_swipe_fingers: u32,
    pub overview_pinch_fingers: u32,
    pub context_hold_fingers: u32,
    pub horizontal_threshold: f64,
    pub pinch_scale_threshold: f64,
}

impl Default for GestureProfile {
    fn default() -> Self {
        Self {
            workspace_swipe_fingers: 3,
            overview_pinch_fingers: 4,
            context_hold_fingers: 1,
            horizontal_threshold: 48.0,
            pinch_scale_threshold: 0.08,
        }
    }
}

impl GestureProfile {
    /// Resolves a completed gesture into a desktop-level semantic action.
    pub fn resolve(
        &self,
        gesture: PointingGesture,
    ) -> Option<PointingInteraction> {
        match gesture {
            PointingGesture::Swipe { fingers, delta_x, delta_y }
                if fingers == self.workspace_swipe_fingers
                    && delta_x.is_finite()
                    && delta_y.is_finite()
                    && delta_x.abs() >= self.horizontal_threshold
                    && delta_x.abs() > delta_y.abs() =>
            {
                if delta_x.is_sign_negative() {
                    Some(PointingInteraction::NextWorkspace)
                } else {
                    Some(PointingInteraction::PreviousWorkspace)
                }
            }
            PointingGesture::Pinch { fingers, scale, .. }
                if fingers == self.overview_pinch_fingers
                    && scale.is_finite()
                    && (scale - 1.0).abs() >= self.pinch_scale_threshold =>
            {
                Some(PointingInteraction::Overview)
            }
            PointingGesture::Hold { fingers }
                if fingers == self.context_hold_fingers =>
            {
                Some(PointingInteraction::ContextMenu)
            }
            _ => None,
        }
    }
}
