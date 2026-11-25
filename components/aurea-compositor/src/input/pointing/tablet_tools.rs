// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Physical ownership and active-source arbitration for tablet tools.

use std::collections::{HashMap, HashSet};

use aurea_desktop_api::input::InputDeviceId;
use smithay::backend::input::TabletToolDescriptor;

/// Result of removing one physical tablet from the tool registry.
#[derive(Debug)]
pub struct RemovedTabletTool {
    pub descriptor: TabletToolDescriptor,
    /// Last event time when this tablet owned the active proximity session.
    pub active_time: Option<u32>,
    pub still_owned: bool,
}

/// Tracks physical tablet ownership separately from the active pen source.
///
/// A uniquely identified pen may be used on multiple tablets over time, but
/// only one physical tablet can be the active source for that pen at a time.
#[derive(Debug, Default)]
pub struct TabletToolRegistry {
    owners: HashMap<InputDeviceId, HashSet<TabletToolDescriptor>>,
    active: HashMap<TabletToolDescriptor, (InputDeviceId, u32)>,
}

impl TabletToolRegistry {
    /// Records that a physical tablet has reported this tool.
    pub fn observe(&mut self, device: InputDeviceId, tool: TabletToolDescriptor) {
        self.owners.entry(device).or_default().insert(tool);
    }

    /// Makes this physical tablet the active source for a tool.
    ///
    /// Returns the previous active source, if another tablet owned the tool.
    pub fn activate(
        &mut self,
        device: InputDeviceId,
        tool: TabletToolDescriptor,
        time: u32,
    ) -> Option<(InputDeviceId, u32)> {
        self.observe(device, tool.clone());
        let previous = self.active.insert(tool, (device, time));
        previous.filter(|(previous_device, _)| *previous_device != device)
    }

    /// Checks whether the tool belongs to the physical tablet.
    pub fn is_owned_by(&self, device: InputDeviceId, tool: &TabletToolDescriptor) -> bool {
        self.owners
            .get(&device)
            .is_some_and(|tools| tools.contains(tool))
    }

    /// Checks whether the tablet currently owns the tool's active proximity.
    pub fn is_active_on(&self, device: InputDeviceId, tool: &TabletToolDescriptor) -> bool {
        self.active
            .get(tool)
            .is_some_and(|(active_device, _)| *active_device == device)
    }

    /// Updates the last event timestamp for the active physical source.
    pub fn update_time(
        &mut self,
        device: InputDeviceId,
        tool: &TabletToolDescriptor,
        time: u32,
    ) -> bool {
        let Some((active_device, last_time)) = self.active.get_mut(tool) else {
            return false;
        };
        if *active_device != device {
            return false;
        }
        *last_time = time;
        true
    }

    /// Ends proximity only when the caller is the active physical source.
    pub fn deactivate(
        &mut self,
        device: InputDeviceId,
        tool: &TabletToolDescriptor,
    ) -> Option<u32> {
        if !self.is_active_on(device, tool) {
            return None;
        }
        self.active.remove(tool).map(|(_, time)| time)
    }

    /// Removes one tablet and reports whether each tool is still used elsewhere.
    pub fn remove_device(&mut self, device: InputDeviceId) -> Vec<RemovedTabletTool> {
        let tools = self.owners.remove(&device).unwrap_or_default();
        tools
            .into_iter()
            .map(|descriptor| {
                let active_time = self.deactivate(device, &descriptor);
                let still_owned = self
                    .owners
                    .values()
                    .any(|owned| owned.contains(&descriptor));
                RemovedTabletTool {
                    descriptor,
                    active_time,
                    still_owned,
                }
            })
            .collect()
    }

    /// Clears all physical ownership and active tool sessions.
    pub fn clear(&mut self) {
        self.owners.clear();
        self.active.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smithay::backend::input::{TabletToolCapabilities, TabletToolType};

    fn tool(serial: u64) -> TabletToolDescriptor {
        TabletToolDescriptor {
            tool_type: TabletToolType::Pen,
            hardware_serial: serial,
            hardware_id_wacom: serial + 100,
            capabilities: TabletToolCapabilities::PRESSURE | TabletToolCapabilities::TILT,
        }
    }

    #[test]
    fn one_tablet_cannot_send_events_for_another_tablets_active_pen() {
        let mut registry = TabletToolRegistry::default();
        let first = InputDeviceId(1);
        let second = InputDeviceId(2);
        let pen = tool(10);

        registry.activate(first, pen.clone(), 100);
        assert!(registry.is_active_on(first, &pen));
        assert!(!registry.is_active_on(second, &pen));
        assert!(!registry.update_time(second, &pen, 200));
    }

    #[test]
    fn same_unique_pen_can_move_between_tablets_without_duplicate_ownership() {
        let mut registry = TabletToolRegistry::default();
        let first = InputDeviceId(3);
        let second = InputDeviceId(4);
        let pen = tool(20);

        registry.activate(first, pen.clone(), 100);
        assert_eq!(registry.activate(second, pen.clone(), 200), Some((first, 100)));
        assert!(!registry.is_active_on(first, &pen));
        assert!(registry.is_active_on(second, &pen));
        assert!(registry.is_owned_by(first, &pen));
        assert!(registry.is_owned_by(second, &pen));
    }

    #[test]
    fn disconnecting_one_tablet_preserves_a_pen_owned_by_another() {
        let mut registry = TabletToolRegistry::default();
        let first = InputDeviceId(5);
        let second = InputDeviceId(6);
        let pen = tool(30);

        registry.observe(first, pen.clone());
        registry.activate(second, pen.clone(), 300);

        let removed = registry.remove_device(first);
        assert_eq!(removed.len(), 1);
        assert!(removed[0].active_time.is_none());
        assert!(removed[0].still_owned);
        assert!(registry.is_active_on(second, &pen));
    }

    #[test]
    fn disconnecting_active_tablet_ends_only_its_tool_session() {
        let mut registry = TabletToolRegistry::default();
        let first = InputDeviceId(7);
        let second = InputDeviceId(8);
        let first_pen = tool(40);
        let second_pen = tool(41);
        registry.activate(first, first_pen.clone(), 400);
        registry.activate(second, second_pen.clone(), 500);

        let removed = registry.remove_device(first);
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].active_time, Some(400));
        assert!(!removed[0].still_owned);
        assert!(registry.is_active_on(second, &second_pen));
    }
}
