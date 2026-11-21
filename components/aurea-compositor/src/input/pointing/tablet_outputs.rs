// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Per-device tablet-to-output mapping policies.

use std::collections::HashMap;

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
}

impl TabletOutputMap {
    /// Creates an empty mapping; unassigned tablets use the full desktop.
    pub fn new() -> Self {
        Self::default()
    }

    /// Assigns a tablet to the full desktop or one output.
    pub fn assign(&mut self, device: InputDeviceId, target: TabletOutputTarget) {
        self.assignments.insert(device, target);
    }

    /// Removes an explicit mapping and restores full-desktop behavior.
    pub fn unassign(&mut self, device: InputDeviceId) -> bool {
        self.assignments.remove(&device).is_some()
    }

    /// Removes mappings that explicitly reference a disconnected output.
    pub fn remove_output(&mut self, output: OutputId) {
        self.assignments.retain(|_, target| {
            !matches!(target, TabletOutputTarget::Output(assigned) if *assigned == output)
        });
    }

    /// Removes the mapping belonging to a disconnected tablet.
    pub fn remove_device(&mut self, device: InputDeviceId) {
        self.assignments.remove(&device);
    }

    /// Resolves the target against the currently enabled output set.
    ///
    /// The full desktop is the default. An explicit output mapping is only
    /// accepted while that output is enabled; it is never redirected.
    pub fn resolve(
        &self,
        device: InputDeviceId,
        enabled_outputs: impl IntoIterator<Item = OutputId>,
    ) -> Option<TabletOutputTarget> {
        let outputs = enabled_outputs.into_iter().collect::<Vec<_>>();
        if outputs.is_empty() {
            return None;
        }

        match self.assignments.get(&device).copied() {
            Some(TabletOutputTarget::Output(output)) => {
                outputs.contains(&output).then_some(TabletOutputTarget::Output(output))
            }
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
        assert_eq!(map.resolve(DEVICE, [FIRST, SECOND]), Some(TabletOutputTarget::All));
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
        assert_eq!(map.resolve(DEVICE, [FIRST, SECOND]), Some(TabletOutputTarget::All));
        assert!(!map.unassign(DEVICE));

        map.assign(DEVICE, TabletOutputTarget::Output(SECOND));
        map.remove_device(DEVICE);
        assert_eq!(map.resolve(DEVICE, [FIRST, SECOND]), Some(TabletOutputTarget::All));
    }

    #[test]
    fn removing_output_clears_only_its_explicit_assignments() {
        let mut map = TabletOutputMap::new();
        map.assign(DEVICE, TabletOutputTarget::Output(SECOND));
        map.remove_output(SECOND);

        assert_eq!(map.resolve(DEVICE, [FIRST]), Some(TabletOutputTarget::All));
    }
}
