// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Normalized gestures and semantic desktop interactions.

/// Lifecycle phase reported by a hardware provider.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GesturePhase {
    Begin,
    Update,
    End { cancelled: bool },
}

/// Gesture data after provider-specific normalization.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointingGesture {
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

/// Desktop-level intent. The shell decides how each intent is executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointingInteraction {
    PreviousWorkspace,
    NextWorkspace,
    Overview,
    ContextMenu,
}
