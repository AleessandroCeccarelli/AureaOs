// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Per-device tablet-to-output mapping policies.

use std::collections::{HashMap, HashSet};

use aurea_desktop_api::input::InputDeviceId;

use crate::output::OutputId;

/// Area of the desktop controlled by one tablet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabletOutputTarget {
    /// Map the tablet across the complete enabled desktop.
    All,
    /// Map the tablet to one enabled output.
    Output(OutputId),
}

/// Stores an optional output target for each physical tablet.
#[derive(Debug, Clone, Default)]
pub struct TabletOutputMap {
    assignments: HashMap<InputDeviceId, TabletOutputTarget>,
    unavailable_devices: HashSet<InputDeviceId>,
}

impl TabletOutputMap {
    /// Creates an empty mapping; unassigned tablets use the full desktop.
    pub fn new() -> Self {
        Self::default()
    }

    /// Assigns a tablet to the full desktop or one output.
    pub fn assign(&mut self, device: InputDeviceId, target: TabletOutputTarget) {
        self.unavailable_devices.remove(&device);
        self.assignments.insert(device, target);
    }

    /// Removes an explicit mapping and restores full-desktop behavior.
    pub fn unassign(&mut self, device: InputDeviceId) -> bool {
        self.unavailable_devices.remove(&device);
        self.assignments.remove(&device).is_some()
    }

    /// Marks tablets assigned to a disconnected display as unavailable.
    ///
    /// The stale target is retained so a later single-output configuration
    /// cannot silently capture the tablet. Reassign or unassign the device
    /// explicitly to restore routing.
    pub fn remove_output(&mut self, output: OutputId) {
        for (device, target) in &self.assignments {
            if matches!(target, TabletOutputTarget::Output(assigned) if *assigned == output) {
                self.unavailable_devices.insert(*device);
            }
        }
    }

    /// Removes the mapping belonging to a disconnected tablet.
    pub fn remove_device(&mut self, device: InputDeviceId) {
        self.assignments.remove(&device);
        self.unavailable_devices.remove(&device);
    }

    /// Resolves the target against the currently enabled output set.
    ///
    /// A stale explicit assignment is never redirected. The full desktop is
    /// the default only when no unavailable explicit assignment exists.
    pub fn resolve(
        &self,
        device: InputDeviceId,
        enabled_outputs: impl IntoIterator<Item = OutputId>,
    ) -> Option<TabletOutputTarget> {
        let outputs = enabled_outputs.into_iter().collect::<Vec<_>>();
        if outputs.is_empty() || self.unavailable_devices.contains(&device) {
            return None;
        }

        match self.assignments.get(&device).copied() {
            Some(TabletOutputTarget::Output(output)) => outputs
                .contains(&output)
                .then_some(TabletOutputTarget::Output(output)),
            Some(TabletOutputTarget::All) | None => Some(TabletOutputTarget::All),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEVICE: InputDeviceId = InputDeviceId(11);
    const FIRST: OutputId = 101;
    const SECOND: OutputId = 202;

    #[test]
    fn defaults_to_all_enabled_outputs() {
        let map = TabletOutputMap::new();
        assert_eq!(
            map.resolve(DEVICE, [FIRST, SECOND]),
            Some(TabletOutputTarget::All)
        );
    }

    #[test]
    fn explicit_output_mapping_is_respected() {
        let mut map = TabletOutputMap::new();
        map.assign(DEVICE, TabletOutputTarget::Output(SECOND));

        assert_eq!(
            map.resolve(DEVICE, [FIRST, SECOND]),
            Some(TabletOutputTarget::Output(SECOND))
        );
    }

    #[test]
    fn unavailable_output_mapping_is_not_redirected() {
        let mut map = TabletOutputMap::new();
        map.assign(DEVICE, TabletOutputTarget::Output(SECOND));

        assert_eq!(map.resolve(DEVICE, [FIRST]), None);
    }

    #[test]
    fn all_mode_requires_at_least_one_enabled_output() {
        let mut map = TabletOutputMap::new();
        map.assign(DEVICE, TabletOutputTarget::All);

        assert_eq!(map.resolve(DEVICE, []), None);
    }

    #[test]
    fn unassign_and_device_removal_restore_default_behavior() {
        let mut map = TabletOutputMap::new();
        map.assign(DEVICE, TabletOutputTarget::Output(SECOND));

        assert!(map.unassign(DEVICE));
        assert_eq!(
            map.resolve(DEVICE, [FIRST, SECOND]),
            Some(TabletOutputTarget::All)
        );
        assert!(!map.unassign(DEVICE));

        map.assign(DEVICE, TabletOutputTarget::Output(SECOND));
        map.remove_device(DEVICE);
        assert_eq!(
            map.resolve(DEVICE, [FIRST, SECOND]),
            Some(TabletOutputTarget::All)
        );
    }

    #[test]
    fn removing_output_blocks_rerouting_until_explicit_action() {
        let mut map = TabletOutputMap::new();
        map.assign(DEVICE, TabletOutputTarget::Output(SECOND));
        map.remove_output(SECOND);

        assert_eq!(map.resolve(DEVICE, [FIRST]), None);

        map.assign(DEVICE, TabletOutputTarget::Output(FIRST));
        assert_eq!(
            map.resolve(DEVICE, [FIRST]),
            Some(TabletOutputTarget::Output(FIRST))
        );

        map.remove_output(FIRST);
        assert_eq!(map.resolve(DEVICE, [SECOND]), None);

        assert!(map.unassign(DEVICE));
        assert_eq!(
            map.resolve(DEVICE, [SECOND]),
            Some(TabletOutputTarget::All)
        );
    }

    #[test]
    fn removing_output_preserves_other_assignments() {
        let mut map = TabletOutputMap::new();
        let other_device = InputDeviceId(12);
        map.assign(DEVICE, TabletOutputTarget::Output(SECOND));
        map.assign(other_device, TabletOutputTarget::Output(FIRST));

        map.remove_output(SECOND);

        assert_eq!(map.resolve(DEVICE, [FIRST]), None);
        assert_eq!(
            map.resolve(other_device, [FIRST]),
            Some(TabletOutputTarget::Output(FIRST))
        );
    }
}
