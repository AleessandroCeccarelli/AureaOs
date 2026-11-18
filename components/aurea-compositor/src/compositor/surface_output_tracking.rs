// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Tracks the Wayland outputs currently associated with each surface.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use smithay::{
    output::Output,
    utils::IsAlive,
    reexports::wayland_server::{
        Resource,
        backend::ObjectId,
        protocol::wl_surface::WlSurface,
    },
};

use crate::output::OutputId;

/// Output membership changes for one surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MembershipChanges {
    pub entered: Vec<OutputId>,
    pub left: Vec<OutputId>,
}

/// Calculates the difference between two output membership sets.
pub(super) fn membership_changes(
    previous: &BTreeSet<OutputId>,
    next: &BTreeSet<OutputId>,
) -> MembershipChanges {
    MembershipChanges {
        entered: next.difference(previous).copied().collect(),
        left: previous.difference(next).copied().collect(),
    }
}

struct TrackedSurface {
    surface: WlSurface,
    outputs: BTreeMap<OutputId, Output>,
}

/// Owns the output membership of every tracked Wayland surface.
///
/// Geometry is deliberately supplied by the caller. This module only applies
/// validated membership changes and emits the corresponding Wayland events.
#[derive(Default)]
pub(super) struct SurfaceOutputTracker {
    surfaces: HashMap<ObjectId, TrackedSurface>,
}

impl SurfaceOutputTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reconciles one surface against the outputs whose logical bounds it
    /// intersects. Outputs omitted from the iterator receive a leave event.
    pub fn reconcile(
        &mut self,
        surface: &WlSurface,
        intersecting_outputs: impl IntoIterator<Item = (OutputId, Output)>,
    ) {
        let surface_id = surface.id();
        let next_outputs = intersecting_outputs
            .into_iter()
            .filter(|(_, output)| output.current_mode().is_some())
            .collect::<BTreeMap<_, _>>();

        let previous_ids = self
            .surfaces
            .get(&surface_id)
            .map(|tracked| tracked.outputs.keys().copied().collect())
            .unwrap_or_default();
        let next_ids = next_outputs.keys().copied().collect();
        let changes = membership_changes(&previous_ids, &next_ids);

        if let Some(previous) = self.surfaces.get(&surface_id) {
            for output_id in changes.left {
                if let Some(output) = previous.outputs.get(&output_id) {
                    output.leave(surface);
                }
            }
        }

        for output_id in changes.entered {
            if let Some(output) = next_outputs.get(&output_id) {
                output.enter(surface);
            }
        }

        if next_outputs.is_empty() {
            self.surfaces.remove(&surface_id);
        } else {
            self.surfaces.insert(
                surface_id,
                TrackedSurface {
                    surface: surface.clone(),
                    outputs: next_outputs,
                },
            );
        }
    }

    /// Removes one surface from all outputs it previously entered.
    pub fn remove_surface(&mut self, surface: &WlSurface) {
        if let Some(tracked) = self.surfaces.remove(&surface.id()) {
            for output in tracked.outputs.values() {
                output.leave(&tracked.surface);
            }
        }
    }

    /// Sends leave events to every surface associated with a removed output.
    pub fn remove_output(&mut self, output_id: OutputId) {
        self.surfaces.retain(|_, tracked| {
            if let Some(output) = tracked.outputs.remove(&output_id) {
                output.leave(&tracked.surface);
            }
            !tracked.outputs.is_empty()
        });
    }

    /// Removes destroyed surfaces and releases their retained output handles.
    pub fn prune_dead_surfaces(&mut self) {
        self.surfaces
            .retain(|_, tracked| tracked.surface.alive());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn membership_diff_reports_only_new_and_removed_outputs() {
        let previous = BTreeSet::from([1, 2, 3]);
        let next = BTreeSet::from([2, 3, 4]);

        assert_eq!(
            membership_changes(&previous, &next),
            MembershipChanges {
                entered: vec![4],
                left: vec![1],
            }
        );
    }

    #[test]
    fn membership_diff_handles_first_entry_and_final_exit() {
        let empty = BTreeSet::new();
        let active = BTreeSet::from([7, 9]);

        assert_eq!(
            membership_changes(&empty, &active),
            MembershipChanges {
                entered: vec![7, 9],
                left: vec![],
            }
        );
        assert_eq!(
            membership_changes(&active, &empty),
            MembershipChanges {
                entered: vec![],
                left: vec![7, 9],
            }
        );
    }

    #[test]
    fn unchanged_membership_does_not_emit_transitions() {
        let active = BTreeSet::from([5, 8]);
        assert_eq!(
            membership_changes(&active, &active),
            MembershipChanges {
                entered: vec![],
                left: vec![],
            }
        );
    }
}
