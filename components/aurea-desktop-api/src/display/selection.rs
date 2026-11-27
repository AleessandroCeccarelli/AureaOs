// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Conservative, hardware-independent display-mode selection.
//!
//! This module selects only modes already reported by the active display
//! capability provider. It never invents refresh rates or infers VRR from
//! connector names or EDID marketing strings.

use super::{
    capabilities::DisplayDescriptor,
    modes::DisplayMode,
    policy::{ActiveRefreshStrategy, DisplayManagementMode, DisplayPolicyContext, DisplayRefreshPolicy},
};

/// Why a display mode could not be selected safely.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayModeSelectionError {
    NoSupportedModes,
    ManualModeRequired,
    RequestedModeUnavailable,
}

/// A supported mode together with the refresh strategy selected for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayModeSelection {
    mode: DisplayMode,
    refresh_strategy: ActiveRefreshStrategy,
}

impl DisplayModeSelection {
    pub const fn mode(self) -> DisplayMode {
        self.mode
    }

    pub const fn refresh_strategy(self) -> ActiveRefreshStrategy {
        self.refresh_strategy
    }
}

/// Selects a mode from the display's verified mode list.
///
/// Manual management requires an exact mode supplied by the caller. Automatic
/// management chooses only from the provider's supported modes. VRR is chosen
/// only when the descriptor contains a verified effective range; otherwise
/// PreferVariable safely falls back to a fixed mode.
pub fn select_display_mode(
    display: &DisplayDescriptor,
    management_mode: DisplayManagementMode,
    requested_mode: Option<DisplayMode>,
    refresh_policy: DisplayRefreshPolicy,
    context: DisplayPolicyContext,
) -> Result<DisplayModeSelection, DisplayModeSelectionError> {
    if display.supported_modes.is_empty() {
        return Err(DisplayModeSelectionError::NoSupportedModes);
    }

    let mode = if management_mode == DisplayManagementMode::Manual {
        let requested = requested_mode.ok_or(DisplayModeSelectionError::ManualModeRequired)?;
        if !display.supported_modes.contains(&requested) {
            return Err(DisplayModeSelectionError::RequestedModeUnavailable);
        }
        requested
    } else {
        select_automatic_mode(display, context)
            .ok_or(DisplayModeSelectionError::NoSupportedModes)?
    };

    let refresh_strategy = match refresh_policy {
        DisplayRefreshPolicy::PreferVariable
            if display.capabilities.variable_refresh.is_some() =>
        {
            ActiveRefreshStrategy::Variable
        }
        DisplayRefreshPolicy::Automatic | DisplayRefreshPolicy::PreferVariable
        | DisplayRefreshPolicy::Fixed => ActiveRefreshStrategy::Fixed,
    };

    Ok(DisplayModeSelection {
        mode,
        refresh_strategy,
    })
}

/// Chooses the highest-resolution mode, then a refresh rate appropriate for
/// the measured frame cadence. If the cadence is unknown, it prefers 60 Hz
/// when that exact rate is available; otherwise it uses the highest supported
/// refresh rate for an active/high-motion workload and the lowest for an idle
/// or battery-powered workload.
fn select_automatic_mode(
    display: &DisplayDescriptor,
    context: DisplayPolicyContext,
) -> Option<DisplayMode> {
    let maximum_area = display
        .supported_modes
        .iter()
        .map(|mode| u64::from(mode.width()) * u64::from(mode.height()))
        .max()?;

    let mut modes = display
        .supported_modes
        .iter()
        .copied()
        .filter(|mode| u64::from(mode.width()) * u64::from(mode.height()) == maximum_area)
        .collect::<Vec<_>>();
    modes.sort_by_key(|mode| mode.refresh_millihz());

    if let Some(cadence) = context.frame_cadence_millihz {
        return modes
            .iter()
            .copied()
            .find(|mode| mode.refresh_millihz() >= cadence)
            .or_else(|| modes.last().copied());
    }

    if !context.on_battery && (context.fullscreen || context.animations_active) {
        return modes.last().copied();
    }

    modes
        .iter()
        .copied()
        .find(|mode| mode.refresh_millihz() == 60_000)
        .or_else(|| modes.first().copied())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::{
        capabilities::{DisplayCapabilities, DisplayConnector, DisplayId, DisplayDescriptor, VariableRefreshRange},
        modes::DisplayMode,
    };

    fn mode(width: u32, height: u32, refresh: u32) -> DisplayMode {
        DisplayMode::new(width, height, refresh).unwrap()
    }

    fn display(vrr: Option<VariableRefreshRange>) -> DisplayDescriptor {
        DisplayDescriptor {
            id: DisplayId(1),
            name: "eDP-1".to_owned(),
            connector: DisplayConnector::EmbeddedDisplayPort,
            connected: true,
            supported_modes: vec![
                mode(1280, 720, 60_000),
                mode(1920, 1080, 60_000),
                mode(1920, 1080, 120_000),
            ],
            current_mode: Some(mode(1920, 1080, 60_000)),
            capabilities: DisplayCapabilities {
                variable_refresh: vrr,
                ..DisplayCapabilities::default()
            },
        }
    }

    #[test]
    fn automatic_selection_prefers_highest_resolution_and_idle_sixty_hz() {
        let selected = select_display_mode(
            &display(None),
            DisplayManagementMode::Automatic,
            None,
            DisplayRefreshPolicy::Automatic,
            DisplayPolicyContext::default(),
        )
        .unwrap();
        assert_eq!(selected.mode(), mode(1920, 1080, 60_000));
        assert_eq!(selected.refresh_strategy(), ActiveRefreshStrategy::Fixed);
    }

    #[test]
    fn active_animation_can_select_the_highest_supported_refresh() {
        let selected = select_display_mode(
            &display(None),
            DisplayManagementMode::Automatic,
            None,
            DisplayRefreshPolicy::Automatic,
            DisplayPolicyContext {
                animations_active: true,
                ..DisplayPolicyContext::default()
            },
        )
        .unwrap();
        assert_eq!(selected.mode(), mode(1920, 1080, 120_000));
    }

    #[test]
    fn measured_frame_cadence_selects_the_lowest_mode_that_can_cover_it() {
        let selected = select_display_mode(
            &display(None),
            DisplayManagementMode::Automatic,
            None,
            DisplayRefreshPolicy::Automatic,
            DisplayPolicyContext {
                frame_cadence_millihz: Some(90_000),
                ..DisplayPolicyContext::default()
            },
        )
        .unwrap();
        assert_eq!(selected.mode(), mode(1920, 1080, 120_000));
    }

    #[test]
    fn manual_mode_requires_an_exact_supported_mode() {
        assert_eq!(
            select_display_mode(
                &display(None),
                DisplayManagementMode::Manual,
                None,
                DisplayRefreshPolicy::Fixed,
                DisplayPolicyContext::default(),
            ),
            Err(DisplayModeSelectionError::ManualModeRequired)
        );
        assert_eq!(
            select_display_mode(
                &display(None),
                DisplayManagementMode::Manual,
                Some(mode(2560, 1440, 60_000)),
                DisplayRefreshPolicy::Fixed,
                DisplayPolicyContext::default(),
            ),
            Err(DisplayModeSelectionError::RequestedModeUnavailable)
        );
    }

    #[test]
    fn variable_refresh_is_used_only_with_a_verified_effective_range() {
        let selected = select_display_mode(
            &display(Some(VariableRefreshRange::new(48_000, 120_000).unwrap())),
            DisplayManagementMode::Automatic,
            None,
            DisplayRefreshPolicy::PreferVariable,
            DisplayPolicyContext::default(),
        )
        .unwrap();
        assert_eq!(selected.refresh_strategy(), ActiveRefreshStrategy::Variable);

        let fallback = select_display_mode(
            &display(None),
            DisplayManagementMode::Automatic,
            None,
            DisplayRefreshPolicy::PreferVariable,
            DisplayPolicyContext::default(),
        )
        .unwrap();
        assert_eq!(fallback.refresh_strategy(), ActiveRefreshStrategy::Fixed);
    }

    #[test]
    fn no_mode_is_selected_when_the_provider_reports_none() {
        let mut descriptor = display(None);
        descriptor.supported_modes.clear();
        assert_eq!(
            select_display_mode(
                &descriptor,
                DisplayManagementMode::Automatic,
                None,
                DisplayRefreshPolicy::Automatic,
                DisplayPolicyContext::default(),
            ),
            Err(DisplayModeSelectionError::NoSupportedModes)
        );
    }
}
