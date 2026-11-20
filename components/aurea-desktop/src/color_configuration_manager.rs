// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Desktop-level coordinator for saved color configurations and display outputs.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::RwLock,
};

use crate::display_color_assignment_store::{
    DisplayColorAssignmentStore, DisplayColorStoreError,
};

use aurea_desktop_api::display::{
    DisplayColorConfigurationAssignment, DisplayColorConfigurationError,
    DisplayColorConfigurationId, DisplayColorConfigurationManager, DisplayDescriptor,
    DisplayEvent, DisplayId,
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
/// again. When opened with a persistent store, assignments are committed
/// atomically and restored across desktop restarts.
#[derive(Debug, Default)]
pub struct DesktopColorConfigurationManager {
    state: RwLock<RegistryState>,
    persistent_store: Option<DisplayColorAssignmentStore>,
}

impl DesktopColorConfigurationManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens the coordinator using the current user's AureaOS config directory.
    pub fn open_default_persistent() -> Result<Self, DisplayColorStoreError> {
        Self::open_persistent(DisplayColorAssignmentStore::user_default()?)
    }

    /// Opens the coordinator with durable per-user display assignments.
    ///
    /// Saved configuration IDs are registered immediately; display IDs become
    /// active only after the display backend rediscovers those outputs.
    pub fn open_persistent(
        store: DisplayColorAssignmentStore,
    ) -> Result<Self, DisplayColorStoreError> {
        let assignments = store.load()?;
        let known_configurations = assignments
            .iter()
            .filter_map(|assignment| assignment.configuration_id.clone())
            .collect();
        let assignments = assignments
            .into_iter()
            .filter_map(|assignment| {
                assignment.configuration_id.map(|configuration_id| {
                    (assignment.display_id, configuration_id)
                })
            })
            .collect();

        Ok(Self {
            state: RwLock::new(RegistryState {
                connected_displays: BTreeSet::new(),
                known_configurations,
                assignments,
            }),
            persistent_store: Some(store),
        })
    }

    fn persist_assignments(
        &self,
        state: &RegistryState,
    ) -> Result<(), DisplayColorConfigurationError> {
        let Some(store) = &self.persistent_store else {
            return Ok(());
        };
        let assignments = state
            .assignments
            .iter()
            .map(|(display_id, configuration_id)| DisplayColorConfigurationAssignment {
                display_id: *display_id,
                configuration_id: Some(configuration_id.clone()),
            })
            .collect::<Vec<_>>();
        store
            .save(&assignments)
            .map_err(|_| DisplayColorConfigurationError::AssignmentRejected)
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

    /// Applies one lifecycle event emitted by the display manager.
    ///
    /// Capability and mode changes do not affect color assignments. A
    /// disconnected output keeps its assignment so it can be restored when
    /// the same stable display identity reconnects.
    pub fn handle_display_event(
        &self,
        event: &DisplayEvent,
    ) -> Result<(), DisplayColorConfigurationError> {
        match event {
            DisplayEvent::Connected(descriptor) if descriptor.connected => {
                self.register_display(descriptor.id)
            }
            DisplayEvent::Connected(descriptor) if !descriptor.connected => {
                self.unregister_display(descriptor.id)
            }
            DisplayEvent::Disconnected(display_id) => self.unregister_display(*display_id),
            DisplayEvent::CapabilitiesChanged(_) 
            | DisplayEvent::ConfigurationChanged
            | DisplayEvent::PrimaryDisplayChanged(_) => Ok(()),
            DisplayEvent::Connected(_) => Ok(()),
        }
    }

    /// Reconciles the connected-output set with a fresh display snapshot.
    ///
    /// Existing assignments are retained for outputs absent from the snapshot.
    pub fn synchronize_displays(
        &self,
        displays: &[DisplayDescriptor],
    ) -> Result<(), DisplayColorConfigurationError> {
        let mut state = self
            .state
            .write()
            .map_err(|_| DisplayColorConfigurationError::AssignmentRejected)?;

        state.connected_displays = displays
            .iter()
            .filter(|display| display.connected)
            .map(|display| display.id)
            .collect();

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

        if let Some(configuration_id) = &assignment.configuration_id {
            if !state.known_configurations.contains(configuration_id) {
                return Err(DisplayColorConfigurationError::ConfigurationNotFound);
            }
        }

        let previous = state.assignments.get(&assignment.display_id).cloned();
        match assignment.configuration_id {
            Some(configuration_id) => {
                state.assignments.insert(assignment.display_id, configuration_id);
            }
            None => {
                state.assignments.remove(&assignment.display_id);
            }
        }

        if self.persist_assignments(&state).is_err() {
            match previous {
                Some(configuration_id) => {
                    state.assignments.insert(assignment.display_id, configuration_id);
                }
                None => {
                    state.assignments.remove(&assignment.display_id);
                }
            }
            return Err(DisplayColorConfigurationError::AssignmentRejected);
        }

        Ok(())
    }

    fn clear(&self, display_id: DisplayId) -> Result<(), DisplayColorConfigurationError> {
        let mut state = self
            .state
            .write()
            .map_err(|_| DisplayColorConfigurationError::AssignmentRejected)?;
        let previous = state.assignments.remove(&display_id);
        if self.persist_assignments(&state).is_err() {
            if let Some(configuration_id) = previous {
                state.assignments.insert(display_id, configuration_id);
            }
            return Err(DisplayColorConfigurationError::AssignmentRejected);
        }
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
    fn persistent_assignments_survive_manager_recreation() {
        let path = std::env::temp_dir().join(format!(
            "aurea-color-manager-{}-{}",
            std::process::id(),
            configuration("persisted").as_str()
        )).join("assignments");
        let store = DisplayColorAssignmentStore::open(&path).unwrap();
        let output = display(55);
        let profile = configuration("persisted");

        {
            let manager = DesktopColorConfigurationManager::open_persistent(store.clone()).unwrap();
            manager.register_display(output).unwrap();
            manager.register_configuration(profile.clone()).unwrap();
            manager.assign(DisplayColorConfigurationAssignment {
                display_id: output,
                configuration_id: Some(profile.clone()),
            }).unwrap();
        }

        let restored = DesktopColorConfigurationManager::open_persistent(store).unwrap();
        assert_eq!(restored.assignments()[0].configuration_id, Some(profile));
        restored.register_display(output).unwrap();
        assert!(restored.assign(DisplayColorConfigurationAssignment {
            display_id: output,
            configuration_id: Some(configuration("persisted")),
        }).is_ok());

        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn display_lifecycle_events_register_and_unregister_outputs() {
        use aurea_desktop_api::display::{DisplayConnector, DisplayDescriptor};

        let manager = DesktopColorConfigurationManager::new();
        let descriptor = DisplayDescriptor {
            id: display(12),
            name: "DP-1".to_owned(),
            connector: DisplayConnector::DisplayPort,
            connected: true,
            supported_modes: Vec::new(),
            current_mode: None,
            capabilities: Default::default(),
        };

        manager.handle_display_event(&DisplayEvent::Connected(descriptor.clone())).unwrap();
        assert!(manager.assign(DisplayColorConfigurationAssignment {
            display_id: descriptor.id,
            configuration_id: None,
        }).is_ok());

        manager.handle_display_event(&DisplayEvent::Disconnected(descriptor.id)).unwrap();
        assert_eq!(
            manager.assign(DisplayColorConfigurationAssignment {
                display_id: descriptor.id,
                configuration_id: None,
            }),
            Err(DisplayColorConfigurationError::DisplayNotFound)
        );
    }

    #[test]
    fn snapshot_reconciliation_does_not_discard_saved_assignments() {
        use aurea_desktop_api::display::{DisplayConnector, DisplayDescriptor};

        let manager = DesktopColorConfigurationManager::new();
        let output = display(21);
        let profile = configuration("calibrated");
        manager.register_configuration(profile.clone()).unwrap();
        manager.register_display(output).unwrap();
        manager.assign(DisplayColorConfigurationAssignment {
            display_id: output,
            configuration_id: Some(profile.clone()),
        }).unwrap();

        let snapshot = [DisplayDescriptor {
            id: display(22),
            name: "HDMI-A-1".to_owned(),
            connector: DisplayConnector::Hdmi,
            connected: true,
            supported_modes: Vec::new(),
            current_mode: None,
            capabilities: Default::default(),
        }];
        manager.synchronize_displays(&snapshot).unwrap();

        assert_eq!(manager.assignments()[0].configuration_id, Some(profile));
        assert_eq!(
            manager.assign(DisplayColorConfigurationAssignment {
                display_id: output,
                configuration_id: None,
            }),
            Err(DisplayColorConfigurationError::DisplayNotFound)
        );
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
