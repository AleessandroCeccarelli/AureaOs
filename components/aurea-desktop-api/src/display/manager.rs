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
    pub enabled: bool,
    pub mode: DisplayMode,
    pub position: DisplayPosition,
    pub scale: DisplayScale,
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

        let active = self.outputs.iter().filter(|output| output.enabled).count();
        if active > MAX_ACTIVE_DISPLAYS {
            return Err(DisplayManagerError::TooManyActiveDisplays {
                maximum: MAX_ACTIVE_DISPLAYS,
            });
        }

        if let Some(primary) = self.layout.primary_display {
            let is_enabled = self
                .outputs
                .iter()
                .any(|output| output.display == primary && output.enabled);
            if !is_enabled {
                return Err(DisplayManagerError::InvalidPrimaryDisplay);
            }
        }

        let mut placement_ids = std::collections::BTreeSet::new();
        for placement in &self.layout.placements {
            if !placement_ids.insert(placement.display) {
                return Err(DisplayManagerError::DuplicateDisplay);
            }
            let Some(output) = self.outputs.iter().find(|output| output.display == placement.display) else {
                return Err(DisplayManagerError::UnknownDisplay);
            };
            if output.enabled != placement.enabled
                || output.position != placement.position
                || output.scale != placement.scale
            {
                return Err(DisplayManagerError::ConfigurationRejected);
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

    fn output(id: u128, enabled: bool) -> DisplayOutputConfiguration {
        DisplayOutputConfiguration {
            display: DisplayId(id),
            enabled,
            mode: DisplayMode::new(1920, 1080, 60_000).unwrap(),
            position: DisplayPosition::new(0, 0),
            scale: DisplayScale::from_percent(100).unwrap(),
            refresh_policy: DisplayRefreshPolicy::Automatic,
        }
    }

    fn configuration(outputs: Vec<DisplayOutputConfiguration>) -> DisplayConfiguration {
        let placements = outputs
            .iter()
            .map(|output| DisplayPlacement {
                display: output.display,
                enabled: output.enabled,
                position: output.position,
                scale: output.scale,
            })
            .collect();
        DisplayConfiguration {
            management_mode: DisplayManagementMode::Automatic,
            layout: DisplayLayout {
                placements,
                primary_display: outputs.iter().find(|output| output.enabled).map(|output| output.display),
            },
            outputs,
        }
    }

    #[test]
    fn accepts_four_active_displays_and_disabled_discovered_outputs() {
        let mut outputs = (1..=4).map(|id| output(id, true)).collect::<Vec<_>>();
        outputs.push(output(5, false));
        assert!(configuration(outputs).validate().is_ok());
    }

    #[test]
    fn rejects_a_fifth_active_display() {
        let outputs = (1..=5).map(|id| output(id, true)).collect();
        assert_eq!(
            configuration(outputs).validate(),
            Err(DisplayManagerError::TooManyActiveDisplays { maximum: 4 })
        );
    }

    #[test]
    fn rejects_duplicate_display_ids() {
        assert_eq!(
            configuration(vec![output(1, true), output(1, false)]).validate(),
            Err(DisplayManagerError::DuplicateDisplay)
        );
    }

    #[test]
    fn rejects_duplicate_layout_placements() {
        let mut config = configuration(vec![output(1, true), output(2, true)]);
        config.layout.placements[1].display = DisplayId(1);
        assert_eq!(config.validate(), Err(DisplayManagerError::DuplicateDisplay));
    }

    #[test]
    fn rejects_a_disabled_primary_display() {
        let mut config = configuration(vec![output(1, false)]);
        config.layout.primary_display = Some(DisplayId(1));
        assert_eq!(
            config.validate(),
            Err(DisplayManagerError::InvalidPrimaryDisplay)
        );
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
