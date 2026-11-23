// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Desktop-level coordinator for saved color configurations and display outputs.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::RwLock,
};

use crate::display_color_assignment_store::{DisplayColorAssignmentStore, DisplayColorStoreError};

use aurea_desktop_api::display::{
    DisplayColorConfigurationAssignment, DisplayColorConfigurationError,
    DisplayColorConfigurationId, DisplayColorConfigurationManager, DisplayDescriptor, DisplayEvent,
    DisplayId,
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
                assignment
                    .configuration_id
                    .map(|configuration_id| (assignment.display_id, configuration_id))
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
            .map(
                |(display_id, configuration_id)| DisplayColorConfigurationAssignment {
                    display_id: *display_id,
                    configuration_id: Some(configuration_id.clone()),
                },
            )
            .collect::<Vec<_>>();
        store
            .save(&assignments)
            .map_err(|_| DisplayColorConfigurationError::AssignmentRejected)
    }

    /// Announces a display discovered by the active display backend.
    pub fn register_display(
        &self,
        display_id: DisplayId,
    ) -> Result<(), DisplayColorConfigurationError> {
        self.state
            .write()
            .map_err(|_| DisplayColorConfigurationError::AssignmentRejected)?
            .connected_displays
            .insert(display_id);
        Ok(())
    }

    /// Marks a display disconnected without discarding its saved assignment.
    pub fn unregister_display(
        &self,
        display_id: DisplayId,
    ) -> Result<(), DisplayColorConfigurationError> {
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
                state
                    .assignments
                    .insert(assignment.display_id, configuration_id);
            }
            None => {
                state.assignments.remove(&assignment.display_id);
            }
        }

        if self.persist_assignments(&state).is_err() {
            match previous {
                Some(configuration_id) => {
                    state
                        .assignments
                        .insert(assignment.display_id, configuration_id);
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
            .map(
                |(display_id, configuration_id)| DisplayColorConfigurationAssignment {
                    display_id: *display_id,
                    configuration_id: Some(configuration_id.clone()),
                },
            )
            .collect()
    }
}
