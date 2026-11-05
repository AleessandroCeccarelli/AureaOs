// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Coordinates output registration and logical layout.

use super::{
    DisplayOutput,
    OutputId,
    OutputRegistry,
    layout::{OutputLayout, OutputPosition},
};

/// Coordinates output descriptions and their logical positions.
#[derive(Debug, Default)]
pub struct OutputManager {
    registry: OutputRegistry,
    layout: OutputLayout,
}

impl OutputManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an output and its position as one operation.
    ///
    /// Returns false if the output ID is already registered.
    pub fn register(
        &mut self,
        output: DisplayOutput,
        position: OutputPosition,
    ) -> bool {
        let id = output.id();

        if !self.registry.register(output) {
            return false;
        }

        // The ID was just registered, so this insertion should succeed.
        if !self.layout.insert(id, position) {
            self.registry.remove(id);
            return false;
        }

        true
    }

    /// Removes an output and its layout position.
    pub fn remove(&mut self, id: OutputId) -> Option<DisplayOutput> {
        let output = self.registry.remove(id)?;

        self.layout.remove(id);

        Some(output)
    }

    pub fn output(&self, id: OutputId) -> Option<&DisplayOutput> {
        self.registry.get(id)
    }

    pub fn output_mut(
        &mut self,
        id: OutputId,
    ) -> Option<&mut DisplayOutput> {
        self.registry.get_mut(id)
    }

    pub fn position(&self, id: OutputId) -> Option<OutputPosition> {
        self.layout.get(id)
    }

    /// Updates the position of a registered output.
    pub fn set_position(
        &mut self,
        id: OutputId,
        position: OutputPosition,
    ) -> bool {
        self.layout.set_position(id, position)
    }

    pub fn len(&self) -> usize {
        self.registry.len()
    }

    pub fn is_empty(&self) -> bool {
        self.registry.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &DisplayOutput> {
        self.registry.iter()
    }

        /// Changes the enabled state of a registered output.
    pub fn set_enabled(
        &mut self,
        id: OutputId,
        enabled: bool,
    ) -> bool {
        let Some(output) = self.registry.get_mut(id) else {
            return false;
        };

        output.set_enabled(enabled);
        true
    }

    /// Changes the current mode of a registered output.
    pub fn set_mode(
        &mut self,
        id: OutputId,
        mode: OutputMode,
    ) -> bool {
        let Some(output) = self.registry.get_mut(id) else {
            return false;
        };

        output.set_mode(mode)
    }

    /// Changes the scale factor of a registered output.
    pub fn set_scale(
        &mut self,
        id: OutputId,
        scale: u32,
    ) -> bool {
        let Some(output) = self.registry.get_mut(id) else {
            return false;
        };

        output.set_scale(scale)
    }
    

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::OutputMode;

    fn sample_output(id: OutputId) -> DisplayOutput {
        let mode = OutputMode::new(1920, 1080, 60_000).unwrap();

        DisplayOutput::new(
            id,
            format!("DISPLAY-{id}"),
            vec![mode],
            mode,
            1,
        )
        .unwrap()
    }

    #[test]
    fn registers_output_with_position() {
        let mut manager = OutputManager::new();
        let position = OutputPosition::new(1920, 0);

        assert!(manager.register(sample_output(1), position));
        assert_eq!(manager.len(), 1);
        assert_eq!(manager.position(1), Some(position));
        assert!(manager.output(1).is_some());
    }

    #[test]
    fn rejects_duplicate_output_ids() {
        let mut manager = OutputManager::new();

        assert!(manager.register(
            sample_output(1),
            OutputPosition::new(0, 0),
        ));
        assert!(!manager.register(
            sample_output(1),
            OutputPosition::new(1920, 0),
        ));

        assert_eq!(manager.len(), 1);
        assert_eq!(
            manager.position(1),
            Some(OutputPosition::new(0, 0)),
        );
    }

    #[test]
    fn removes_output_and_position_together() {
        let mut manager = OutputManager::new();
        manager.register(
            sample_output(1),
            OutputPosition::new(0, 0),
        );

        assert!(manager.remove(1).is_some());
        assert!(manager.output(1).is_none());
        assert_eq!(manager.position(1), None);
        assert!(manager.is_empty());
    }

    #[test]
    fn updates_only_registered_output_positions() {
        let mut manager = OutputManager::new();
        let position = OutputPosition::new(1920, 0);

        assert!(!manager.set_position(1, position));
        manager.register(
            sample_output(1),
            OutputPosition::default(),
        );

        assert!(manager.set_position(1, position));
        assert_eq!(manager.position(1), Some(position));
    }
}
