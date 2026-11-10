
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Registration, configuration and reconciliation of display outputs.

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fmt,
};

use super::layout::{OutputLayout, OutputPosition};
use super::{DisplayOutput, OutputId, OutputMode, OutputRegistry};

/// Errors returned by output management operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputManagerError {
    DuplicateOutput(OutputId),
    UnknownOutput(OutputId),
    InvalidMode {
        output: OutputId,
        mode: OutputMode,
    },
    InvalidScale {
        output: OutputId,
        scale: u32,
    },
    PositionOverflow,
}

impl fmt::Display for OutputManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateOutput(output) => {
                write!(f, "Output {output} is already registered")
            }
            Self::UnknownOutput(output) => {
                write!(f, "Unknown output: {output}")
            }
            Self::InvalidMode { output, mode } => {
                write!(f, "Mode {mode:?} is not supported by output {output}")
            }
            Self::InvalidScale { output, scale } => {
                write!(f, "Invalid scale {scale} for output {output}")
            }
            Self::PositionOverflow => {
                f.write_str("Output layout position overflow")
            }
        }
    }
}

impl std::error::Error for OutputManagerError {}

/// IDs affected by a per-device output reconciliation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OutputReconciliation {
    pub added: Vec<OutputId>,
    pub updated: Vec<OutputId>,
    pub removed: Vec<OutputId>,
}

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
    ///
    /// All IDs are validated before mutation. If an insertion unexpectedly
    /// fails, outputs inserted by this call are removed before returning.
    pub fn register_batch(
        &mut self,
        outputs: Vec<(DisplayOutput, (i32, i32))>,
    ) -> Result<(), OutputManagerError> {
        let mut batch_ids = BTreeSet::new();

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

    /// Reconciles outputs owned by one DRM device with a fresh discovery result.
    ///
    /// Unchanged outputs keep their object and layout position. Updated outputs
    /// retain scale, enabled state, position and a still-supported current mode.
    /// New outputs are placed after the rightmost output in the final set.
    pub fn reconcile_outputs(
        &mut self,
        owned_ids: &[OutputId],
        discovered: Vec<DisplayOutput>,
    ) -> Result<OutputReconciliation, OutputManagerError> {
        let owned = owned_ids.iter().copied().collect::<HashSet<_>>();
        let mut desired = BTreeSet::new();

        for output in &discovered {
            let id = output.id();

            if !desired.insert(id) {
                return Err(OutputManagerError::DuplicateOutput(id));
            }

            if self.registry.get(id).is_some() && !owned.contains(&id) {
                return Err(OutputManagerError::DuplicateOutput(id));
            }
        }

        let discovered_ids = discovered
            .iter()
            .map(DisplayOutput::id)
            .collect::<HashSet<_>>();

        let mut changes = OutputReconciliation::default();
        let mut updates = Vec::new();
        let mut additions = Vec::new();
        let mut effective_widths = HashMap::new();

        for output in discovered {
            let id = output.id();

            if let Some(current) = self.registry.get(id) {
                let mut replacement = current.clone();
                replacement.update_from_discovery(output);

                effective_widths.insert(id, replacement.current_mode().width());

                if *current != replacement {
                    changes.updated.push(id);
                    updates.push((id, replacement));
                }
            } else {
                changes.added.push(id);
                effective_widths.insert(id, output.current_mode().width());
                additions.push(output);
            }
        }

        changes.removed = owned
            .into_iter()
            .filter(|id| {
                !discovered_ids.contains(id) && self.registry.get(*id).is_some()
            })
            .collect();

        let removed = changes.removed.iter().copied().collect::<HashSet<_>>();

        let mut next_x = self.registry.iter().try_fold(0_i32, |right, output| {
            if removed.contains(&output.id()) {
                return Ok(right);
            }

            let position = self.layout.get(output.id()).unwrap_or_default();

            let width = effective_widths
                .get(&output.id())
                .copied()
                .unwrap_or_else(|| output.current_mode().width());

            let width =
                i32::try_from(width).map_err(|_| OutputManagerError::PositionOverflow)?;

            let edge = position
                .x()
                .checked_add(width)
                .ok_or(OutputManagerError::PositionOverflow)?;

            Ok::<i32, OutputManagerError>(right.max(edge))
        })?;

        let mut addition_positions = Vec::with_capacity(additions.len());

        for output in &additions {
            let width = i32::try_from(output.current_mode().width())
                .map_err(|_| OutputManagerError::PositionOverflow)?;

            addition_positions.push((output.id(), (next_x, 0)));

            next_x = next_x
                .checked_add(width)
                .ok_or(OutputManagerError::PositionOverflow)?;
        }

        // All validation and position arithmetic is complete before mutation.
        let before = self.clone();

        for id in &changes.removed {
            self.unregister(*id);
        }

        for (id, output) in updates {
            if let Some(current) = self.registry.get_mut(id) {
                *current = output;
            }
        }

        for (output, (_, position)) in additions.into_iter().zip(addition_positions) {
            if let Err(error) = self.register(output, position) {
                *self = before;
                return Err(error);
            }
        }

        Ok(changes)
    }

    /// Unregisters an output and removes it from the layout.
    pub fn unregister(&mut self, output_id: OutputId) -> bool {
        self.layout.remove(output_id);
        self.registry.remove(output_id).is_some()
    }

    /// Enables or disables an output.
    pub fn set_enabled(
        &mut self,
        output_id: OutputId,
        enabled: bool,
    ) -> Result<(), OutputManagerError> {
        let output = self
            .registry
            .get_mut(output_id)
            .ok_or(OutputManagerError::UnknownOutput(output_id))?;

        output.set_enabled(enabled);
        Ok(())
    }

    /// Changes the active mode of an output.
    pub fn set_mode(
        &mut self,
        output_id: OutputId,
        mode: OutputMode,
    ) -> Result<(), OutputManagerError> {
        let output = self
            .registry
            .get_mut(output_id)
            .ok_or(OutputManagerError::UnknownOutput(output_id))?;

        if !output.set_mode(mode) {
            return Err(OutputManagerError::InvalidMode {
                output: output_id,
                mode,
            });
        }

        Ok(())
    }

    /// Changes the scale of an output.
    pub fn set_scale(
        &mut self,
        output_id: OutputId,
        scale: u32,
    ) -> Result<(), OutputManagerError> {
        if scale == 0 {
            return Err(OutputManagerError::InvalidScale {
                output: output_id,
                scale,
            });
        }

        let output = self
            .registry
            .get_mut(output_id)
            .ok_or(OutputManagerError::UnknownOutput(output_id))?;

        if !output.set_scale(scale) {
            return Err(OutputManagerError::InvalidScale {
                output: output_id,
                scale,
            });
        }

        Ok(())
    }

    /// Updates the logical position of an output.
    pub fn set_position(
        &mut self,
        output_id: OutputId,
        position: (i32, i32),
    ) -> Result<(), OutputManagerError> {
        if self.registry.get(output_id).is_none() {
            return Err(OutputManagerError::UnknownOutput(output_id));
        }

        self.layout
            .set_position(output_id, OutputPosition::new(position.0, position.1));

        Ok(())
    }
}
