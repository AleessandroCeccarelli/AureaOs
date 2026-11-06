// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli
//! Logical positioning of display outputs.
use std::collections::BTreeMap;
use super::OutputId;
/// Position of an output in compositor logical coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OutputPosition {
    x: i32,
    y: i32,
}
impl OutputPosition {
    /// Creates a logical position.
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    pub const fn x(self) -> i32 {
        self.x
    }
    pub const fn y(self) -> i32 {
        self.y
    }
}
/// Stores logical positions for registered outputs.
#[derive(Debug, Default)]
pub struct OutputLayout {
    positions: BTreeMap<OutputId, OutputPosition>,
}
impl OutputLayout {
    pub fn new() -> Self {
        Self::default()
    }
    /// Adds an output position. Returns false if the ID already exists.
    pub fn insert(&mut self, id: OutputId, position: OutputPosition) -> bool {
        if self.positions.contains_key(&id) {
            return false;
        }
        self.positions.insert(id, position);
        true
    }
    /// Updates an existing output position.
    pub fn set_position(&mut self, id: OutputId, position: OutputPosition) -> bool {
        let Some(current) = self.positions.get_mut(&id) else {
            return false;
        };
        *current = position;
        true
    }
    /// Removes and returns an output position.
    pub fn remove(&mut self, id: OutputId) -> Option<OutputPosition> {
        self.positions.remove(&id)
    }
    pub fn get(&self, id: OutputId) -> Option<OutputPosition> {
        self.positions.get(&id).copied()
    }
    pub fn contains(&self, id: OutputId) -> bool {
        self.positions.contains_key(&id)
    }
    pub fn len(&self) -> usize {
        self.positions.len()
    }
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = (OutputId, OutputPosition)> + '_ {
        self.positions
            .iter()
            .map(|(&id, &position)| (id, position))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inserts_and_reads_positions() {
        let mut layout = OutputLayout::new();
        let position = OutputPosition::new(1920, 0);
        assert!(layout.insert(2, position));
        assert_eq!(layout.get(2), Some(position));
        assert_eq!(layout.len(), 1);
    }
    #[test]
    fn rejects_duplicate_ids() {
        let mut layout = OutputLayout::new();
        assert!(layout.insert(1, OutputPosition::new(0, 0)));
        assert!(!layout.insert(1, OutputPosition::new(1920, 0)));
        assert_eq!(layout.len(), 1);
    }
    #[test]
    fn updates_existing_positions_only() {
        let mut layout = OutputLayout::new();
        let position = OutputPosition::new(1920, 0);
        assert!(!layout.set_position(1, position));
        assert!(layout.insert(1, OutputPosition::default()));
        assert!(layout.set_position(1, position));
        assert_eq!(layout.get(1), Some(position));
    }
}
