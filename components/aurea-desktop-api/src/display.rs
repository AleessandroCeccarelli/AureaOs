// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Shared display color and display-management contracts.

pub mod capabilities;
pub mod events;
pub mod icc;
pub mod layout;
pub mod limits;
pub mod manager;
pub mod modes;
pub mod policy;

pub use capabilities::{
    DisplayCapabilities, DisplayConnector, DisplayDescriptor, DisplayId,
    DisplayPhysicalSize, VariableRefreshRange,
};
pub use events::DisplayEvent;
pub use icc::{
    DisplayIccProfileAssignment, IccProfileCatalogSnapshot, IccProfileClass, IccProfileDescriptor,
    IccProfileFormat, IccProfileImport, IccProfileManager, IccProfileManagerError,
    IccProfileProvider, MAX_ICC_PROFILE_BYTES,
    IccProfileId, IccProfileOrigin,
};
pub use layout::{DisplayLayout, DisplayPlacement, DisplayPosition, DisplayScale};
pub use limits::MAX_ACTIVE_DISPLAYS;
pub use manager::{
    DisplayConfiguration, DisplayManager, DisplayManagerError, DisplayManagerSnapshot,
    DisplayOutputConfiguration,
};
pub use modes::DisplayMode;
pub use policy::{
    ActiveRefreshStrategy, DisplayManagementMode, DisplayPolicyContext, DisplayRefreshPolicy,
};

use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

/// A technology-independent color adjustment requested for a display output.
///
/// The compositor owns the implementation. A value of zero means that the
/// output must remain unchanged; the range is inclusive from 0 to 100.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DisplayColorAdjustment {
    warmth_percent: u8,
}

impl DisplayColorAdjustment {
    /// Creates an adjustment, clamping its intensity to the supported range.
    pub const fn new(warmth_percent: u8) -> Self {
        Self {
            warmth_percent: if warmth_percent > 100 {
                100
            } else {
                warmth_percent
            },
        }
    }

    /// Returns the requested warmth intensity in the range 0–100.
    pub const fn warmth_percent(self) -> u8 {
        self.warmth_percent
    }

    /// Returns whether no color adjustment is requested.
    pub const fn is_neutral(self) -> bool {
        self.warmth_percent == 0
    }
}

/// Thread-safe, technology-independent channel for live display color policy.
///
/// The shell owns updates; the compositor reads snapshots on its render
/// thread. Atomic storage avoids locks in the frame path and keeps the API
/// independent from the graphics backend.
#[derive(Debug, Clone, Default)]
pub struct DisplayColorController {
    warmth_percent: Arc<AtomicU8>,
}

impl DisplayColorController {
    /// Creates a controller with neutral output color.
    pub fn new() -> Self {
        Self::default()
    }

    /// Publishes a new output color adjustment.
    pub fn set_adjustment(&self, adjustment: DisplayColorAdjustment) {
        self.warmth_percent
            .store(adjustment.warmth_percent(), Ordering::Release);
    }

    /// Reads the most recently published adjustment.
    pub fn adjustment(&self) -> DisplayColorAdjustment {
        DisplayColorAdjustment::new(self.warmth_percent.load(Ordering::Acquire))
    }
}

#[cfg(test)]
mod tests {
    use super::{DisplayColorAdjustment, DisplayColorController};

    #[test]
    fn default_adjustment_is_neutral() {
        let adjustment = DisplayColorAdjustment::default();
        assert!(adjustment.is_neutral());
        assert_eq!(adjustment.warmth_percent(), 0);
    }

    #[test]
    fn intensity_is_clamped_to_the_supported_range() {
        assert_eq!(DisplayColorAdjustment::new(65).warmth_percent(), 65);
        assert_eq!(DisplayColorAdjustment::new(u8::MAX).warmth_percent(), 100);
    }

    #[test]
    fn cloned_controller_shares_updates() {
        let controller = DisplayColorController::new();
        let compositor_view = controller.clone();

        controller.set_adjustment(DisplayColorAdjustment::new(68));

        assert_eq!(compositor_view.adjustment().warmth_percent(), 68);
    }

    #[test]
    fn controller_starts_neutral_and_can_be_reset() {
        let controller = DisplayColorController::new();
        assert!(controller.adjustment().is_neutral());

        controller.set_adjustment(DisplayColorAdjustment::new(90));
        controller.set_adjustment(DisplayColorAdjustment::default());

        assert!(controller.adjustment().is_neutral());
    }
}
