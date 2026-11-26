// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Per-device output assignment for absolute pointer devices.

use crate::output::OutputId;
use aurea_desktop_api::input::InputDeviceId;
use std::collections::{HashMap, HashSet};

/// Resolves absolute pointer coordinates against an assigned display.
#[derive(Debug, Clone, Default)]
pub struct AbsolutePointerOutputMap {
    assignments: HashMap<InputDeviceId, OutputId>,
    unavailable_devices: HashSet<InputDeviceId>,
}

impl AbsolutePointerOutputMap {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn assign(&mut self, device: InputDeviceId, output: OutputId) {
        self.unavailable_devices.remove(&device);
        self.assignments.insert(device, output);
    }
    pub fn unassign(&mut self, device: InputDeviceId) -> bool {
        self.unavailable_devices.remove(&device);
        self.assignments.remove(&device).is_some()
    }
    pub fn remove_output(&mut self, output: OutputId) {
        for (device, assigned) in &self.assignments {
            if *assigned == output {
                self.unavailable_devices.insert(*device);
            }
        }
    }
    pub fn remove_device(&mut self, device: InputDeviceId) {
        self.assignments.remove(&device);
        self.unavailable_devices.remove(&device);
    }
    pub fn resolve(
        &self,
        device: InputDeviceId,
        enabled_outputs: impl IntoIterator<Item = OutputId>,
    ) -> Option<OutputId> {
        let outputs = enabled_outputs.into_iter().collect::<Vec<_>>();
        if self.unavailable_devices.contains(&device) {
            return None;
        }
        if let Some(output) = self.assignments.get(&device) {
            return outputs.contains(output).then_some(*output);
        }
        (outputs.len() == 1).then(|| outputs[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const DEVICE: InputDeviceId = InputDeviceId(19);
    const FIRST: OutputId = 10;
    const SECOND: OutputId = 20;

    #[test]
    fn defaults_only_when_one_output_is_enabled() {
        let map = AbsolutePointerOutputMap::new();
        assert_eq!(map.resolve(DEVICE, [FIRST]), Some(FIRST));
        assert_eq!(map.resolve(DEVICE, [FIRST, SECOND]), None);
        assert_eq!(map.resolve(DEVICE, []), None);
    }
    #[test]
    fn explicit_mapping_and_stale_output_are_safe() {
        let mut map = AbsolutePointerOutputMap::new();
        map.assign(DEVICE, SECOND);
        assert_eq!(map.resolve(DEVICE, [FIRST, SECOND]), Some(SECOND));
        map.remove_output(SECOND);
        assert_eq!(map.resolve(DEVICE, [FIRST]), None);
        map.assign(DEVICE, FIRST);
        assert_eq!(map.resolve(DEVICE, [FIRST]), Some(FIRST));
    }
    #[test]
    fn unassign_and_disconnect_clear_mapping() {
        let mut map = AbsolutePointerOutputMap::new();
        map.assign(DEVICE, SECOND);
        assert!(map.unassign(DEVICE));
        assert_eq!(map.resolve(DEVICE, [FIRST]), Some(FIRST));
        map.assign(DEVICE, SECOND);
        map.remove_device(DEVICE);
        assert_eq!(map.resolve(DEVICE, [FIRST]), Some(FIRST));
    }
}
