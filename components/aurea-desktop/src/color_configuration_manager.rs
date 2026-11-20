// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Desktop-level coordinator for saved color configurations and display outputs.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::RwLock,
};

use aurea_desktop_api::display::{
    DisplayColorConfigurationAssignment, DisplayColorConfigurationError,
    DisplayColorConfigurationId, DisplayColorConfigurationManager, DisplayId,
};

#[derive(Debug, Default)]
struct RegistryState {
    connected_displays: BTreeSet<DisplayId>,
    known_configurations: BTreeSet<DisplayColorConfigurationId>,
    assignments: BTreeMap<DisplayId, DisplayColorConfigurationId>,
}

/// Thread-safe coordinator shared by desktop services and settings adapters.
///
/// Assignments survive display disconnects for the lifetime of this registry;
/// reconnecting the same stable DisplayId makes the saved selection available
/// again. Durable persistence is deliberately delegated to a later storage
/// adapter rather than being mixed into this in-memory coordinator.
#[derive(Debug, Default)]
pub struct DesktopColorConfigurationManager {
    state: RwLock<RegistryState>,
}

impl DesktopColorConfigurationManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Announces a display discovered by the active display backend.
    pub fn register_display(&self, display_id: DisplayId) -> Result<(), DisplayColorConfigurationError> {
        self.state
            .write()
            .map_err(|_| DisplayColorConfigurationError::AssignmentRejected)?
            .connected_displays
            .insert(display_id);
        Ok(())
    }

    /// Marks a display disconnected without discarding its saved assignment.
    pub fn unregister_display(&self, display_id: DisplayId) -> Result<(), DisplayColorConfigurationError> {
        self.state
            .write()
            .map_err(|_| DisplayColorConfigurationError::AssignmentRejected)?
            .connected_displays
            .remove(&display_id);
        Ok(())
    }

    /// Makes a configuration eligible for display assignment.
    pub fn register_configuration(
        &self,
        configuration_id: DisplayColorConfigurationId,
    ) -> Result<(), DisplayColorConfigurationError> {
        self.state
            .write()
            .map_err(|_| DisplayColorConfigurationError::AssignmentRejected)?
            .known_configurations
            .insert(configuration_id);
        Ok(())
    }

    /// Removes a configuration only when no display references it.
    pub fn unregister_configuration(
        &self,
        configuration_id: &DisplayColorConfigurationId,
    ) -> Result<bool, DisplayColorConfigurationError> {
        let mut state = self
            .state
            .write()
            .map_err(|_| DisplayColorConfigurationError::AssignmentRejected)?;

        if state.assignments.values().any(|id| id == configuration_id) {
            return Ok(false);
        }

        Ok(state.known_configurations.remove(configuration_id))
    }
}

impl DisplayColorConfigurationManager for DesktopColorConfigurationManager {
    fn assign(
        &self,
        assignment: DisplayColorConfigurationAssignment,
    ) -> Result<(), DisplayColorConfigurationError> {
        let mut state = self
            .state
            .write()
            .map_err(|_| DisplayColorConfigurationError::AssignmentRejected)?;

        if !state.connected_displays.contains(&assignment.display_id) {
            return Err(DisplayColorConfigurationError::DisplayNotFound);
        }

        match assignment.configuration_id {
            Some(configuration_id) => {
                if !state.known_configurations.contains(&configuration_id) {
                    return Err(DisplayColorConfigurationError::ConfigurationNotFound);
                }
                state.assignments.insert(assignment.display_id, configuration_id);
            }
            None => {
                state.assignments.remove(&assignment.display_id);
            }
        }

        Ok(())
    }

    fn clear(&self, display_id: DisplayId) -> Result<(), DisplayColorConfigurationError> {
        self.state
            .write()
            .map_err(|_| DisplayColorConfigurationError::AssignmentRejected)?
            .assignments
            .remove(&display_id);
        Ok(())
    }

    fn assignments(&self) -> Vec<DisplayColorConfigurationAssignment> {
        let Ok(state) = self.state.read() else {
            return Vec::new();
        };

        state
            .assignments
            .iter()
            .map(|(display_id, configuration_id)| DisplayColorConfigurationAssignment {
                display_id: *display_id,
                configuration_id: Some(configuration_id.clone()),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurea_desktop_api::display::DisplayColorConfigurationManager;

    fn display(value: u128) -> DisplayId {
        DisplayId(value)
    }

    fn configuration(value: &str) -> DisplayColorConfigurationId {
        DisplayColorConfigurationId::new(value).unwrap()
    }

    #[test]
    fn assignments_are_independent_per_display() {
        let manager = DesktopColorConfigurationManager::new();
        let first = display(1);
        let second = display(2);
        let warm = configuration("warm");
        let neutral = configuration("neutral");

        manager.register_display(first).unwrap();
        manager.register_display(second).unwrap();
        manager.register_configuration(warm.clone()).unwrap();
        manager.register_configuration(neutral.clone()).unwrap();

        manager.assign(DisplayColorConfigurationAssignment {
            display_id: first,
            configuration_id: Some(warm.clone()),
        }).unwrap();
        manager.assign(DisplayColorConfigurationAssignment {
            display_id: second,
            configuration_id: Some(neutral.clone()),
        }).unwrap();

        let assignments = manager.assignments();
        assert_eq!(assignments.len(), 2);
        assert_eq!(
            assignments.iter().find(|item| item.display_id == first).unwrap().configuration_id,
            Some(warm)
        );
        assert_eq!(
            assignments.iter().find(|item| item.display_id == second).unwrap().configuration_id,
            Some(neutral)
        );
    }

    #[test]
    fn disconnected_display_keeps_assignment_for_reconnection() {
        let manager = DesktopColorConfigurationManager::new();
        let output = display(7);
        let profile = configuration("photo");
        manager.register_display(output).unwrap();
        manager.register_configuration(profile.clone()).unwrap();
        manager.assign(DisplayColorConfigurationAssignment {
            display_id: output,
            configuration_id: Some(profile.clone()),
        }).unwrap();

        manager.unregister_display(output).unwrap();
        assert_eq!(
            manager.assign(DisplayColorConfigurationAssignment {
                display_id: output,
                configuration_id: Some(profile.clone()),
            }),
            Err(DisplayColorConfigurationError::DisplayNotFound)
        );
        assert_eq!(manager.assignments().len(), 1);

        manager.register_display(output).unwrap();
        assert_eq!(manager.assignments()[0].configuration_id, Some(profile));
    }

    #[test]
    fn rejects_unknown_displays_and_configurations() {
        let manager = DesktopColorConfigurationManager::new();
        let output = display(3);
        manager.register_display(output).unwrap();

        assert_eq!(
            manager.assign(DisplayColorConfigurationAssignment {
                display_id: output,
                configuration_id: Some(configuration("missing")),
            }),
            Err(DisplayColorConfigurationError::ConfigurationNotFound)
        );
        assert_eq!(
            manager.assign(DisplayColorConfigurationAssignment {
                display_id: display(99),
                configuration_id: None,
            }),
            Err(DisplayColorConfigurationError::DisplayNotFound)
        );
    }

    #[test]
    fn referenced_configuration_cannot_be_removed() {
        let manager = DesktopColorConfigurationManager::new();
        let output = display(4);
        let profile = configuration("studio");
        manager.register_display(output).unwrap();
        manager.register_configuration(profile.clone()).unwrap();
        manager.assign(DisplayColorConfigurationAssignment {
            display_id: output,
            configuration_id: Some(profile.clone()),
        }).unwrap();

        assert_eq!(manager.unregister_configuration(&profile), Ok(false));
        manager.clear(output).unwrap();
        assert_eq!(manager.unregister_configuration(&profile), Ok(true));
    }
}
