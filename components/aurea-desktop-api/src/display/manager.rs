// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Transactional display configuration service contract.

use std::sync::mpsc::Receiver;

use super::{
    capabilities::{DisplayDescriptor, DisplayId},
    events::DisplayEvent,
    layout::{DisplayLayout, DisplayPlacement, DisplayPosition, DisplayScale},
    limits::MAX_ACTIVE_DISPLAYS,
    modes::DisplayMode,
    policy::{DisplayManagementMode, DisplayRefreshPolicy},
};

/// Configuration requested for one display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayOutputConfiguration {
    pub display: DisplayId,
    /// None lets Automatic management select a supported mode.
    pub mode: Option<DisplayMode>,
    pub refresh_policy: DisplayRefreshPolicy,
}

/// Complete configuration submitted as one atomic operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayConfiguration {
    pub management_mode: DisplayManagementMode,
    pub layout: DisplayLayout,
    pub outputs: Vec<DisplayOutputConfiguration>,
}

/// Current display inventory and effective configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayManagerSnapshot {
    pub displays: Vec<DisplayDescriptor>,
    pub configuration: DisplayConfiguration,
}

/// Failures returned by the display service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayManagerError {
    TooManyActiveDisplays { maximum: usize },
    DuplicateDisplay,
    UnknownDisplay,
    InvalidPrimaryDisplay,
    InvalidMode,
    UnsupportedRefreshPolicy,
    ConfigurationRejected,
    ApplyFailed,
    RollbackFailed,
}

impl DisplayConfiguration {
    /// Validates invariants independent from hardware/backend capabilities.
    pub fn validate(&self) -> Result<(), DisplayManagerError> {
        let mut ids = std::collections::BTreeSet::new();
        for output in &self.outputs {
            if !ids.insert(output.display) {
                return Err(DisplayManagerError::DuplicateDisplay);
            }
        }

        let active = self
            .layout
            .placements
            .iter()
            .filter(|placement| placement.enabled)
            .count();
        if active > MAX_ACTIVE_DISPLAYS {
            return Err(DisplayManagerError::TooManyActiveDisplays {
                maximum: MAX_ACTIVE_DISPLAYS,
            });
        }

        match (active, self.layout.primary_display) {
            (0, None) => {}
            (0, Some(_)) => return Err(DisplayManagerError::InvalidPrimaryDisplay),
            (_, None) => return Err(DisplayManagerError::InvalidPrimaryDisplay),
            (_, Some(primary)) => {
                let is_enabled = self
                    .layout
                    .placements
                    .iter()
                    .any(|placement| placement.display == primary && placement.enabled);
                if !is_enabled {
                    return Err(DisplayManagerError::InvalidPrimaryDisplay);
                }
            }
        }

        for output in &self.outputs {
            if (self.management_mode == DisplayManagementMode::Manual && output.mode.is_none())
                || (output.refresh_policy == DisplayRefreshPolicy::Fixed && output.mode.is_none())
            {
                return Err(DisplayManagerError::InvalidMode);
            }
        }

        let mut placement_ids = std::collections::BTreeSet::new();
        for placement in &self.layout.placements {
            if !placement_ids.insert(placement.display) {
                return Err(DisplayManagerError::DuplicateDisplay);
            }
            if !self
                .outputs
                .iter()
                .any(|output| output.display == placement.display)
            {
                return Err(DisplayManagerError::UnknownDisplay);
            }
        }

        if placement_ids != ids {
            return Err(DisplayManagerError::ConfigurationRejected);
        }

        Ok(())
    }
}

/// Trusted service that owns display inventory and configuration.
///
/// Implementations must validate the complete request before changing state.
/// Applying a configuration is transactional: on failure, restore the last
/// known working configuration and report rollback failure separately.
/// The service must never claim VRR availability unless the active driver and
/// complete presentation pipeline have verified it.
pub trait DisplayManager: Send + Sync {
    fn snapshot(&self) -> DisplayManagerSnapshot;
    fn apply_configuration(
        &self,
        configuration: DisplayConfiguration,
    ) -> Result<(), DisplayManagerError>;
    fn subscribe(&self) -> Receiver<DisplayEvent>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::capabilities::DisplayConnector;

    fn output(id: u128) -> DisplayOutputConfiguration {
        DisplayOutputConfiguration {
            display: DisplayId(id),
            mode: Some(DisplayMode::new(1920, 1080, 60_000).unwrap()),
            refresh_policy: DisplayRefreshPolicy::Automatic,
        }
    }

    fn configuration(outputs: Vec<DisplayOutputConfiguration>) -> DisplayConfiguration {
        let placements = outputs
            .iter()
            .map(|output| DisplayPlacement {
                display: output.display,
                enabled: true,
                position: DisplayPosition::new(0, 0),
                scale: DisplayScale::from_percent(100).unwrap(),
            })
            .collect();
        DisplayConfiguration {
            management_mode: DisplayManagementMode::Automatic,
            layout: DisplayLayout {
                placements,
                primary_display: outputs.first().map(|output| output.display),
            },
            outputs,
        }
    }

    #[test]
    fn accepts_four_active_displays_and_disabled_discovered_outputs() {
        let outputs = (1..=(MAX_ACTIVE_DISPLAYS as u128 + 1))
            .map(|id| output(id))
            .collect();
        let mut config = configuration(outputs);
        config.layout.placements[MAX_ACTIVE_DISPLAYS].enabled = false;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn rejects_a_fifth_active_display() {
        let outputs = (1..=(MAX_ACTIVE_DISPLAYS as u128 + 1))
            .map(output)
            .collect();
        assert_eq!(
            configuration(outputs).validate(),
            Err(DisplayManagerError::TooManyActiveDisplays {
                maximum: MAX_ACTIVE_DISPLAYS,
            })
        );
    }

    #[test]
    fn rejects_duplicate_display_ids() {
        assert_eq!(
            configuration(vec![output(1), output(1)]).validate(),
            Err(DisplayManagerError::DuplicateDisplay)
        );
    }

    #[test]
    fn rejects_duplicate_layout_placements() {
        let mut config = configuration(vec![output(1), output(2)]);
        config.layout.placements[1].display = DisplayId(1);
        assert_eq!(
            config.validate(),
            Err(DisplayManagerError::DuplicateDisplay)
        );
    }

    #[test]
    fn rejects_a_disabled_primary_display() {
        let mut config = configuration(vec![output(1)]);
        config.layout.placements[0].enabled = false;
        assert_eq!(
            config.validate(),
            Err(DisplayManagerError::InvalidPrimaryDisplay)
        );
    }

    #[test]
    fn automatic_mode_may_defer_mode_selection_to_the_policy_engine() {
        let mut config = configuration(vec![output(1)]);
        config.outputs[0].mode = None;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn manual_mode_requires_an_explicit_supported_mode() {
        let mut config = configuration(vec![output(1, true)]);
        config.management_mode = DisplayManagementMode::Manual;
        config.outputs[0].mode = None;
        assert_eq!(config.validate(), Err(DisplayManagerError::InvalidMode));
    }

    #[test]
    fn capability_descriptors_are_independent_from_compositor_types() {
        let descriptor = DisplayDescriptor {
            id: DisplayId(1),
            name: "DP-1".to_owned(),
            connector: DisplayConnector::DisplayPort,
            connected: true,
            supported_modes: vec![DisplayMode::new(1920, 1080, 60_000).unwrap()],
            current_mode: Some(DisplayMode::new(1920, 1080, 60_000).unwrap()),
            capabilities: Default::default(),
        };
        assert!(!descriptor.supports_variable_refresh());
    }
}
