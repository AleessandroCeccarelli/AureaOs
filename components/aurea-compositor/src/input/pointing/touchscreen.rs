// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Direct-touch display device detection and monitor assignment.

use std::collections::{HashMap, HashSet};

use aurea_desktop_api::input::InputDeviceId;
use smithay::reexports::input::{Device, DeviceCapability};

use crate::output::OutputId;

/// Resolves each touchscreen to the display it physically controls.
///
/// A single active output is a safe automatic default. With multiple outputs,
/// an explicit device-to-output assignment is required to avoid sending touch
/// coordinates to the wrong monitor.
#[derive(Debug, Clone, Default)]
pub struct TouchscreenOutputMap {
    assignments: HashMap<InputDeviceId, OutputId>,
    unavailable_devices: HashSet<InputDeviceId>,
}

impl TouchscreenOutputMap {
    /// Creates an empty touchscreen assignment map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Assigns a touchscreen to a specific output.
    pub fn assign(&mut self, device: InputDeviceId, output: OutputId) {
        self.unavailable_devices.remove(&device);
        self.assignments.insert(device, output);
    }

    /// Removes an explicit assignment for a touchscreen.
    pub fn unassign(&mut self, device: InputDeviceId) -> bool {
        self.unavailable_devices.remove(&device);
        self.assignments.remove(&device).is_some()
    }

    /// Marks devices assigned to a disconnected display as unavailable.
    ///
    /// The stale target is retained so a later single-output configuration
    /// cannot silently capture the touchscreen. Reassign or unassign the
    /// device explicitly to restore routing.
    pub fn remove_output(&mut self, output: OutputId) {
        for (device, assigned) in &self.assignments {
            if *assigned == output {
                self.unavailable_devices.insert(*device);
            }
        }
    }

    /// Removes every assignment belonging to a disconnected device.
    pub fn remove_device(&mut self, device: InputDeviceId) {
        self.assignments.remove(&device);
        self.unavailable_devices.remove(&device);
    }

    /// Resolves an assignment against the currently enabled outputs.
    ///
    /// A stale explicit assignment is never silently redirected. If no
    /// assignment exists, automatic routing is allowed only for one output.
    pub fn resolve(
        &self,
        device: InputDeviceId,
        enabled_outputs: impl IntoIterator<Item = OutputId>,
    ) -> Option<OutputId> {
        let outputs = enabled_outputs.into_iter().collect::<Vec<_>>();
        if self.unavailable_devices.contains(&device) {
            return None;
        }
        if let Some(assigned) = self.assignments.get(&device) {
            return outputs.contains(assigned).then_some(*assigned);
        }

        (outputs.len() == 1).then(|| outputs[0])
    }
}

/// Returns whether a libinput device exposes direct-touch input.
pub(super) fn supports(device: &Device) -> bool {
    device.has_capability(DeviceCapability::Touch)
        && !device.has_capability(DeviceCapability::TabletTool)
        && !device.has_capability(DeviceCapability::TabletPad)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEVICE: InputDeviceId = InputDeviceId(7);
    const FIRST_OUTPUT: OutputId = 101;
    const SECOND_OUTPUT: OutputId = 202;

    #[test]
    fn automatically_routes_only_when_one_output_is_enabled() {
        let mapping = TouchscreenOutputMap::new();

        assert_eq!(mapping.resolve(DEVICE, [FIRST_OUTPUT]), Some(FIRST_OUTPUT));
        assert_eq!(mapping.resolve(DEVICE, [FIRST_OUTPUT, SECOND_OUTPUT]), None);
        assert_eq!(mapping.resolve(DEVICE, []), None);
    }

    #[test]
    fn explicit_assignment_selects_its_output() {
        let mut mapping = TouchscreenOutputMap::new();
        mapping.assign(DEVICE, SECOND_OUTPUT);

        assert_eq!(
            mapping.resolve(DEVICE, [FIRST_OUTPUT, SECOND_OUTPUT]),
            Some(SECOND_OUTPUT)
        );
    }

    #[test]
    fn stale_assignment_is_not_redirected_to_another_monitor() {
        let mut mapping = TouchscreenOutputMap::new();
        mapping.assign(DEVICE, SECOND_OUTPUT);

        assert_eq!(mapping.resolve(DEVICE, [FIRST_OUTPUT]), None);
    }

    #[test]
    fn removing_output_blocks_automatic_rerouting_until_reassigned() {
        let mut mapping = TouchscreenOutputMap::new();
        mapping.assign(DEVICE, SECOND_OUTPUT);
        mapping.remove_output(SECOND_OUTPUT);

        assert_eq!(mapping.resolve(DEVICE, [FIRST_OUTPUT]), None);

        mapping.assign(DEVICE, FIRST_OUTPUT);
        mapping.remove_device(DEVICE);

        assert_eq!(mapping.resolve(DEVICE, [FIRST_OUTPUT, SECOND_OUTPUT]), None);
    }

    #[test]
    fn unassign_reports_whether_a_mapping_existed() {
        let mut mapping = TouchscreenOutputMap::new();
        mapping.assign(DEVICE, FIRST_OUTPUT);

        assert!(mapping.unassign(DEVICE));
        assert!(!mapping.unassign(DEVICE));
    }
}
