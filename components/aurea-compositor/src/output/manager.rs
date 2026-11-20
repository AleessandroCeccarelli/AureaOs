// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Coordinates output registration and the shared output registry.

use std::collections::BTreeSet;

use super::layout::{OutputLayout, OutputPosition};
use super::limits::MAX_ACTIVE_OUTPUTS;
use super::{DisplayOutput, OutputId, OutputRegistry};

pub mod configuration;
pub mod errors;
pub mod geometry;
pub mod reconciliation;

pub use configuration::{OutputConfiguration, OutputScale};
pub use errors::{OutputManagerError, OutputReconciliation};

/// Coordinates output registration, layout and configuration.
#[derive(Debug, Clone, Default)]
pub struct OutputManager {
    registry: OutputRegistry,
    layout: OutputLayout,
}

impl OutputManager {
    /// Creates an empty output manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the output registry.
    pub fn registry(&self) -> &OutputRegistry {
        &self.registry
    }

    /// Returns the output layout.
    pub fn layout(&self) -> &OutputLayout {
        &self.layout
    }

    /// Returns the number of enabled outputs.
    pub fn active_output_count(&self) -> usize {
        self.registry
            .iter()
            .filter(|output| output.is_enabled())
            .count()
    }

    fn ensure_active_capacity(&self, additional: usize) -> Result<(), OutputManagerError> {
        if self.active_output_count().saturating_add(additional) > MAX_ACTIVE_OUTPUTS {
            return Err(OutputManagerError::TooManyActiveOutputs {
                maximum: MAX_ACTIVE_OUTPUTS,
            });
        }

        Ok(())
    }

    /// Registers an output and places it in the layout.
    pub fn register(
        &mut self,
        output: DisplayOutput,
        position: (i32, i32),
    ) -> Result<(), OutputManagerError> {
        let id = output.id();

        if self.registry.get(id).is_some() {
            return Err(OutputManagerError::DuplicateOutput(id));
        }
        if output.is_enabled() {
            self.ensure_active_capacity(1)?;
        }
        if !self.registry.register(output) {
            return Err(OutputManagerError::DuplicateOutput(id));
        }

        let inserted = self
            .layout
            .insert(id, OutputPosition::new(position.0, position.1));
        if !inserted {
            self.registry.remove(id);
            return Err(OutputManagerError::DuplicateOutput(id));
        }

        Ok(())
    }

    /// Registers a group of outputs atomically.
    pub fn register_batch(
        &mut self,
        outputs: Vec<(DisplayOutput, (i32, i32))>,
    ) -> Result<(), OutputManagerError> {
        let mut batch_ids = BTreeSet::new();
        let additional_active = outputs
            .iter()
            .filter(|(output, _)| output.is_enabled())
            .count();

        self.ensure_active_capacity(additional_active)?;

        for (output, _) in &outputs {
            let id = output.id();
            if self.registry.get(id).is_some() || !batch_ids.insert(id) {
                return Err(OutputManagerError::DuplicateOutput(id));
            }
        }

        let mut registered_ids = Vec::with_capacity(outputs.len());
        for (output, position) in outputs {
            let id = output.id();
            if let Err(error) = self.register(output, position) {
                for registered_id in registered_ids {
                    self.unregister(registered_id);
                }
                return Err(error);
            }
            registered_ids.push(id);
        }

        Ok(())
    }

    /// Unregisters an output and removes it from the layout.
    pub fn unregister(&mut self, output_id: OutputId) -> bool {
        self.layout.remove(output_id);
        self.registry.remove(output_id).is_some()
    }


}

#[cfg(test)]
#[path = "manager_tests.rs"]
mod tests;
