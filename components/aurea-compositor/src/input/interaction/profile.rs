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
    pub overview_swipe_fingers: u32,
    pub app_expose_swipe_fingers: u32,
    pub context_hold_fingers: u32,
    pub horizontal_threshold: f64,
    pub vertical_threshold: f64,
    pub pinch_scale_threshold: f64,
}

impl Default for GestureProfile {
    fn default() -> Self {
        Self {
            workspace_swipe_fingers: 3,
            overview_pinch_fingers: 4,
            overview_swipe_fingers: 3,
            app_expose_swipe_fingers: 3,
            context_hold_fingers: 1,
            horizontal_threshold: 48.0,
            vertical_threshold: 48.0,
            pinch_scale_threshold: 0.08,
        }
    }
}

impl GestureProfile {
    /// Resolves a completed gesture into a desktop-level semantic action.
    pub fn resolve(&self, gesture: PointingGesture) -> Option<PointingInteraction> {
        match gesture {
            PointingGesture::Swipe {
                fingers,
                delta_x,
                delta_y,
            } if fingers == self.workspace_swipe_fingers
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
            PointingGesture::Swipe { fingers, delta_x, delta_y }
                if fingers == self.overview_swipe_fingers
                    && delta_x.is_finite()
                    && delta_y.is_finite()
                    && delta_y.abs() >= self.vertical_threshold
                    && delta_y.abs() > delta_x.abs()
                    && delta_y.is_sign_negative() =>
            {
                Some(PointingInteraction::Overview)
            }
            PointingGesture::Swipe { fingers, delta_x, delta_y }
                if fingers == self.app_expose_swipe_fingers
                    && delta_x.is_finite()
                    && delta_y.is_finite()
                    && delta_y >= self.vertical_threshold
                    && delta_y.abs() > delta_x.abs() =>
            {
                Some(PointingInteraction::AppExpose)
            }
            PointingGesture::Pinch { fingers, scale, .. }
                if fingers == self.overview_pinch_fingers
                    && scale.is_finite()
                    && (scale - 1.0).abs() >= self.pinch_scale_threshold =>
            {
                Some(PointingInteraction::Overview)
            }
            PointingGesture::Hold { fingers } if fingers == self.context_hold_fingers => {
                Some(PointingInteraction::ContextMenu)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GestureProfile, PointingGesture, PointingInteraction};

    #[test]
    fn horizontal_swipes_resolve_in_both_directions() {
        let profile = GestureProfile::default();
        assert_eq!(
            profile.resolve(PointingGesture::Swipe {
                fingers: 3,
                delta_x: -60.0,
                delta_y: 2.0,
            }),
            Some(PointingInteraction::NextWorkspace)
        );
        assert_eq!(
            profile.resolve(PointingGesture::Swipe {
                fingers: 3,
                delta_x: 60.0,
                delta_y: 2.0,
            }),
            Some(PointingInteraction::PreviousWorkspace)
        );
    }

    #[test]
    fn vertical_or_short_swipes_do_not_switch_workspaces() {
        let profile = GestureProfile::default();
        assert_ne!(
            profile.resolve(PointingGesture::Swipe {
                fingers: 3,
                delta_x: 10.0,
                delta_y: 80.0,
            }),
            Some(PointingInteraction::NextWorkspace)
        );
        assert_eq!(
            profile.resolve(PointingGesture::Swipe {
                fingers: 3,
                delta_x: 20.0,
                delta_y: 1.0,
            }),
            None
        );
    }

    #[test]
    fn three_finger_up_swipe_opens_overview() {
        let profile = GestureProfile::default();
        assert_eq!(
            profile.resolve(PointingGesture::Swipe {
                fingers: 3,
                delta_x: 2.0,
                delta_y: -64.0,
            }),
            Some(PointingInteraction::Overview)
        );
    }

    #[test]
    fn three_finger_down_swipe_opens_app_expose() {
        let profile = GestureProfile::default();
        assert_eq!(
            profile.resolve(PointingGesture::Swipe {
                fingers: 3,
                delta_x: 1.0,
                delta_y: 72.0,
            }),
            Some(PointingInteraction::AppExpose)
        );
    }

    #[test]
    fn short_or_diagonal_downward_swipes_do_not_open_app_expose() {
        let profile = GestureProfile::default();
        assert_eq!(
            profile.resolve(PointingGesture::Swipe {
                fingers: 3,
                delta_x: 0.0,
                delta_y: 20.0,
            }),
            None
        );
        assert_ne!(
            profile.resolve(PointingGesture::Swipe {
                fingers: 3,
                delta_x: 80.0,
                delta_y: 60.0,
            }),
            Some(PointingInteraction::AppExpose)
        );
    }

    #[test]
    fn downward_or_short_vertical_swipes_do_not_open_overview() {
        let profile = GestureProfile::default();
        assert_ne!(
            profile.resolve(PointingGesture::Swipe {
                fingers: 3,
                delta_x: 0.0,
                delta_y: 64.0,
            }),
            Some(PointingInteraction::Overview)
        );
        assert_eq!(
            profile.resolve(PointingGesture::Swipe {
                fingers: 3,
                delta_x: 0.0,
                delta_y: -20.0,
            }),
            None
        );
    }

    #[test]
    fn four_finger_pinch_and_single_finger_hold_have_defaults() {
        let profile = GestureProfile::default();
        assert_eq!(
            profile.resolve(PointingGesture::Pinch {
                fingers: 4,
                scale: 0.8,
                rotation: 0.0,
            }),
            Some(PointingInteraction::Overview)
        );
        assert_eq!(
            profile.resolve(PointingGesture::Hold { fingers: 1 }),
            Some(PointingInteraction::ContextMenu)
        );
    }

    #[test]
    fn invalid_gesture_samples_are_ignored() {
        let profile = GestureProfile::default();
        assert_eq!(
            profile.resolve(PointingGesture::Swipe {
                fingers: 3,
                delta_x: f64::NAN,
                delta_y: 0.0,
            }),
            None
        );
    }
}
