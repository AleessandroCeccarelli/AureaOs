// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! User preferences and policy choices for display refresh behavior.

/// Requested refresh strategy for one display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayRefreshPolicy {
    /// AureaOS chooses a suitable mode/strategy from verified capabilities.
    #[default]
    Automatic,
    /// Use the fixed refresh rate represented by the selected display mode.
    Fixed,
    /// Prefer VRR when the complete active pipeline confirms it is usable.
    PreferVariable,
}

/// User-facing display management mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayManagementMode {
    /// AureaOS selects safe settings using verified capabilities and context.
    #[default]
    Automatic,
    /// Respect explicit user-selected mode and refresh policy.
    Manual,
}

/// Whether a display is currently using a variable or fixed refresh path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveRefreshStrategy {
    Fixed,
    Variable,
    Unknown,
}

/// System context supplied to a future display policy engine.
///
/// Frame cadence is represented in millihertz. Missing measurements must be
/// treated as unknown; policy must not inspect application pixels or content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DisplayPolicyContext {
    pub frame_cadence_millihz: Option<u32>,
    pub fullscreen: bool,
    pub animations_active: bool,
    pub on_battery: bool,
}

#[cfg(test)]
mod tests {
    use super::{DisplayManagementMode, DisplayRefreshPolicy};

    #[test]
    fn defaults_to_automatic_management_and_refresh() {
        assert_eq!(DisplayManagementMode::default(), DisplayManagementMode::Automatic);
        assert_eq!(DisplayRefreshPolicy::default(), DisplayRefreshPolicy::Automatic);
    }
}
