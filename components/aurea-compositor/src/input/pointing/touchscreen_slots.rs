// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Translate per-device libinput touch slots into unique Wayland seat slots.

use std::collections::HashMap;

use aurea_desktop_api::input::InputDeviceId;
use smithay::backend::input::TouchSlot;

/// Assigns a unique seat-wide slot to each physical-device touch point.
///
/// libinput slot identifiers are scoped to their originating device. A single
/// Wayland seat therefore needs a translation layer when several touchscreens
/// are active simultaneously. Released seat slots are recycled so a long-lived
/// session cannot exhaust its slot counter through ordinary touch activity.
#[derive(Debug, Default)]
pub struct TouchscreenSlotMap {
    active: HashMap<(InputDeviceId, TouchSlot), u32>,
    free_slots: Vec<u32>,
    next_slot: u32,
}

impl TouchscreenSlotMap {
    /// Creates an empty slot map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the existing seat slot or allocates a new one.
    pub fn acquire(&mut self, device: InputDeviceId, slot: TouchSlot) -> Option<TouchSlot> {
        if let Some(id) = self.active.get(&(device, slot)) {
            return Some(TouchSlot::from(Some(*id)));
        }

        let id = match self.free_slots.pop() {
            Some(id) => id,
            None => {
                let id = self.next_slot;
                self.next_slot = self.next_slot.checked_add(1)?;
                id
            }
        };
        self.active.insert((device, slot), id);
        Some(TouchSlot::from(Some(id)))
    }

    /// Returns the seat slot for an already active physical contact.
    pub fn get(&self, device: InputDeviceId, slot: TouchSlot) -> Option<TouchSlot> {
        self.active
            .get(&(device, slot))
            .copied()
            .map(|id| TouchSlot::from(Some(id)))
    }

    /// Releases one touch point after its up event.
    pub fn release(&mut self, device: InputDeviceId, slot: TouchSlot) -> Option<TouchSlot> {
        let id = self.active.remove(&(device, slot))?;
        self.free_slots.push(id);
        Some(TouchSlot::from(Some(id)))
    }

    /// Releases all active touch points belonging to a disconnected device.
    pub fn remove_device(&mut self, device: InputDeviceId) -> Vec<TouchSlot> {
        let keys = self
            .active
            .keys()
            .filter(|(active_device, _)| *active_device == device)
            .copied()
            .collect::<Vec<_>>();

        let released = keys
            .into_iter()
            .filter_map(|key| self.active.remove(&key))
            .collect::<Vec<_>>();
        self.free_slots.extend(released.iter().copied());
        released
            .into_iter()
            .map(|id| TouchSlot::from(Some(id)))
            .collect()
    }

    /// Clears every active contact after a true seat-wide Wayland cancellation.
    /// Device-specific cancellation should use `remove_device` and release only
    /// the returned seat slots, preserving contacts owned by other devices.
    pub fn clear_all(&mut self) -> usize {
        let released = self.active.drain().map(|(_, id)| id).collect::<Vec<_>>();
        let cleared = released.len();
        self.free_slots.extend(released);
        cleared
    }

    /// Returns the number of currently active physical touch points.
    pub fn active_count(&self) -> usize {
        self.active.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(id: u32) -> TouchSlot {
        TouchSlot::from(Some(id))
    }

    #[test]
    fn identical_physical_slots_from_different_devices_are_distinct() {
        let mut slots = TouchscreenSlotMap::new();

        let first = slots.acquire(InputDeviceId(1), slot(0)).unwrap();
        let second = slots.acquire(InputDeviceId(2), slot(0)).unwrap();

        assert_ne!(i32::from(first), i32::from(second));
        assert_eq!(slots.active_count(), 2);
    }

    #[test]
    fn repeated_acquisition_keeps_the_same_wayland_slot() {
        let mut slots = TouchscreenSlotMap::new();

        let first = slots.acquire(InputDeviceId(1), slot(4)).unwrap();
        let second = slots.acquire(InputDeviceId(1), slot(4)).unwrap();

        assert_eq!(first, second);
        assert_eq!(slots.active_count(), 1);
    }

    #[test]
    fn released_seat_slots_are_reused() {
        let mut slots = TouchscreenSlotMap::new();
        let device = InputDeviceId(1);

        let first = slots.acquire(device, slot(0)).unwrap();
        assert_eq!(slots.release(device, slot(0)), Some(first));

        let second = slots.acquire(device, slot(1)).unwrap();
        assert_eq!(first, second);
        assert_eq!(slots.active_count(), 1);
    }

    #[test]
    fn release_allows_a_new_touch_to_receive_a_fresh_slot_when_not_recycled() {
        let mut slots = TouchscreenSlotMap::new();
        let device = InputDeviceId(1);

        let first = slots.acquire(device, slot(0)).unwrap();
        let second = slots.acquire(device, slot(1)).unwrap();
        assert_eq!(slots.release(device, slot(0)), Some(first));

        let third = slots.acquire(device, slot(2)).unwrap();
        assert_eq!(third, first);
        assert_ne!(third, second);
    }

    #[test]
    fn removing_a_device_releases_only_its_touch_points() {
        let mut slots = TouchscreenSlotMap::new();
        slots.acquire(InputDeviceId(1), slot(0)).unwrap();
        slots.acquire(InputDeviceId(1), slot(1)).unwrap();
        let other = slots.acquire(InputDeviceId(2), slot(0)).unwrap();

        assert_eq!(slots.remove_device(InputDeviceId(1)).len(), 2);
        assert_eq!(slots.active_count(), 1);
        assert_eq!(slots.get(InputDeviceId(2), slot(0)), Some(other));

        let recycled = slots.acquire(InputDeviceId(2), slot(1)).unwrap();
        assert_ne!(recycled, other);
        assert_eq!(slots.active_count(), 2);
    }

    #[test]
    fn seat_wide_cancel_clears_contacts_and_recycles_their_slots() {
        let mut slots = TouchscreenSlotMap::new();
        let first = slots.acquire(InputDeviceId(1), slot(0)).unwrap();
        let second = slots.acquire(InputDeviceId(2), slot(0)).unwrap();

        assert_eq!(slots.clear_all(), 2);
        assert_eq!(slots.active_count(), 0);
        assert_eq!(slots.get(InputDeviceId(1), slot(0)), None);
        assert_eq!(slots.get(InputDeviceId(2), slot(0)), None);

        let recycled_first = slots.acquire(InputDeviceId(3), slot(0)).unwrap();
        let recycled_second = slots.acquire(InputDeviceId(4), slot(0)).unwrap();
        assert_ne!(recycled_first, recycled_second);
        assert!(
            [first, second].contains(&recycled_first) && [first, second].contains(&recycled_second)
        );
    }
}
