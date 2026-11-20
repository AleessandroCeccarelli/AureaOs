// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Registration, configuration and reconciliation of display outputs.

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fmt,
};

use super::coordinates::{LogicalPoint, LogicalSize, OutputLogicalBounds};
use super::layout::{OutputLayout, OutputPosition};
use super::limits::MAX_ACTIVE_OUTPUTS;
use super::{DisplayOutput, OutputId, OutputMode, OutputRegistry};

/// Errors returned by output management operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputManagerError {
    DuplicateOutput(OutputId),
    UnknownOutput(OutputId),
    InvalidMode { output: OutputId, mode: OutputMode },
    InvalidScale { output: OutputId, scale: u32 },
    TooManyActiveOutputs { maximum: usize },
    PositionOverflow,
}

impl fmt::Display for OutputManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateOutput(output) => write!(f, "Output {output} is already registered"),
            Self::UnknownOutput(output) => write!(f, "Unknown output: {output}"),
            Self::InvalidMode { output, mode } => {
                write!(f, "Mode {mode:?} is not supported by output {output}")
            }
            Self::InvalidScale { output, scale } => {
                write!(f, "Invalid scale {scale} for output {output}")
            }
            Self::TooManyActiveOutputs { maximum } => {
                write!(
                    f,
                    "AureaOS supports at most {maximum} active display outputs"
                )
            }
            Self::PositionOverflow => f.write_str("Output layout position overflow"),
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

/// Scale change requested as part of an output configuration transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputScale {
    Integer(u32),
    Fractional(super::scale::ScaleFactor),
}

/// Optional changes applied together to one output.
///
/// Unspecified properties retain their current values.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OutputConfiguration {
    pub mode: Option<OutputMode>,
    pub scale: Option<OutputScale>,
    pub transform: Option<smithay::utils::Transform>,
    pub enabled: Option<bool>,
    pub position: Option<(i32, i32)>,
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

    /// Returns the enabled output's half-open bounds in global logical space.
    ///
    /// The dimensions use the configured fractional scale when present, or
    /// the legacy integer scale otherwise. Disabled outputs have no active
    /// bounds; unknown IDs are reported as errors.
    pub fn logical_bounds(
        &self,
        output_id: OutputId,
    ) -> Result<Option<OutputLogicalBounds>, OutputManagerError> {
        let output = self
            .registry
            .get(output_id)
            .ok_or(OutputManagerError::UnknownOutput(output_id))?;

        if !output.is_enabled() {
            return Ok(None);
        }

        let position = self
            .layout
            .get(output_id)
            .ok_or(OutputManagerError::UnknownOutput(output_id))?;
        let mode = output.current_mode();
        let (physical_width, physical_height) = if swaps_dimensions(output.transform()) {
            (mode.height(), mode.width())
        } else {
            (mode.width(), mode.height())
        };
        let size = LogicalSize {
            width: logical_extent(output, physical_width)?,
            height: logical_extent(output, physical_height)?,
        };
        let origin = LogicalPoint {
            x: i64::from(position.x()),
            y: i64::from(position.y()),
        };

        OutputLogicalBounds::from_logical_size(origin, size)
            .map(Some)
            .ok_or(OutputManagerError::PositionOverflow)
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
                effective_widths.insert(id, logical_width(&replacement)?);
                if *current != replacement {
                    changes.updated.push(id);
                    updates.push((id, replacement));
                }
            } else {
                changes.added.push(id);
                effective_widths.insert(id, logical_width(&output)?);
                additions.push(output);
            }
        }

        changes.removed = owned
            .into_iter()
            .filter(|id| !discovered_ids.contains(id) && self.registry.get(*id).is_some())
            .collect();

        let removed = changes.removed.iter().copied().collect::<HashSet<_>>();
        let retained_active = self
            .registry
            .iter()
            .filter(|output| output.is_enabled() && !removed.contains(&output.id()))
            .count();
        let added_active = additions
            .iter()
            .filter(|output| output.is_enabled())
            .count();
        if retained_active.saturating_add(added_active) > MAX_ACTIVE_OUTPUTS {
            return Err(OutputManagerError::TooManyActiveOutputs {
                maximum: MAX_ACTIVE_OUTPUTS,
            });
        }

        let mut next_x = self.registry.iter().try_fold(0_i32, |right, output| {
            if removed.contains(&output.id()) {
                return Ok(right);
            }
            let position = self.layout.get(output.id()).unwrap_or_default();
            let width = effective_widths
                .get(&output.id())
                .copied()
                .map(Ok)
                .unwrap_or_else(|| logical_width(output))?;
            let edge = position
                .x()
                .checked_add(width)
                .ok_or(OutputManagerError::PositionOverflow)?;
            Ok::<i32, OutputManagerError>(right.max(edge))
        })?;

        let mut addition_positions = Vec::with_capacity(additions.len());
        for output in &additions {
            let width = logical_width(output)?;
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

    /// Applies a group of output changes atomically.
    ///
    /// All requested properties are staged on a cloned manager. The active
    /// output limit and resulting logical bounds are validated before the
    /// staged state replaces the current configuration.
    pub fn configure_output(
        &mut self,
        output_id: OutputId,
        configuration: OutputConfiguration,
    ) -> Result<(), OutputManagerError> {
        let mut staged = self.clone();
        let output = staged
            .registry
            .get_mut(output_id)
            .ok_or(OutputManagerError::UnknownOutput(output_id))?;

        if let Some(mode) = configuration.mode {
            if !output.set_mode(mode) {
                return Err(OutputManagerError::InvalidMode {
                    output: output_id,
                    mode,
                });
            }
        }

        if let Some(scale) = configuration.scale {
            match scale {
                OutputScale::Integer(scale) => {
                    if !output.set_scale(scale) {
                        return Err(OutputManagerError::InvalidScale {
                            output: output_id,
                            scale,
                        });
                    }
                }
                OutputScale::Fractional(scale) => output.set_fractional_scale(scale),
            }
        }

        if let Some(transform) = configuration.transform {
            output.set_transform(transform);
        }

        if let Some(enabled) = configuration.enabled {
            output.set_enabled(enabled);
        }

        if let Some(position) = configuration.position {
            staged.layout.set_position(
                output_id,
                OutputPosition::new(position.0, position.1),
            );
        }

        if staged.active_output_count() > MAX_ACTIVE_OUTPUTS {
            return Err(OutputManagerError::TooManyActiveOutputs {
                maximum: MAX_ACTIVE_OUTPUTS,
            });
        }

        // Validate dimensions, scale conversion and global logical bounds
        // against the complete staged configuration before committing it.
        staged.logical_bounds(output_id)?;
        *self = staged;
        Ok(())
    }

    /// Enables or disables an output after validating the resulting layout.
    pub fn set_enabled(
        &mut self,
        output_id: OutputId,
        enabled: bool,
    ) -> Result<(), OutputManagerError> {
        self.configure_output(
            output_id,
            OutputConfiguration {
                enabled: Some(enabled),
                ..OutputConfiguration::default()
            },
        )
    }

    /// Changes the active mode after validating the resulting logical bounds.
    pub fn set_mode(
        &mut self,
        output_id: OutputId,
        mode: OutputMode,
    ) -> Result<(), OutputManagerError> {
        self.configure_output(
            output_id,
            OutputConfiguration {
                mode: Some(mode),
                ..OutputConfiguration::default()
            },
        )
    }

    /// Changes the integer scale after validating the resulting logical bounds.
    pub fn set_scale(&mut self, output_id: OutputId, scale: u32) -> Result<(), OutputManagerError> {
        self.configure_output(
            output_id,
            OutputConfiguration {
                scale: Some(OutputScale::Integer(scale)),
                ..OutputConfiguration::default()
            },
        )
    }

    /// Changes the fractional scale after validating the resulting logical bounds.
    pub fn set_fractional_scale(
        &mut self,
        output_id: OutputId,
        scale: super::scale::ScaleFactor,
    ) -> Result<(), OutputManagerError> {
        self.configure_output(
            output_id,
            OutputConfiguration {
                scale: Some(OutputScale::Fractional(scale)),
                ..OutputConfiguration::default()
            },
        )
    }

    /// Updates the logical position after validating the resulting output bounds.
    pub fn set_position(
        &mut self,
        output_id: OutputId,
        position: (i32, i32),
    ) -> Result<(), OutputManagerError> {
        self.configure_output(
            output_id,
            OutputConfiguration {
                position: Some(position),
                ..OutputConfiguration::default()
            },
        )
    }
}

fn logical_width(output: &DisplayOutput) -> Result<i32, OutputManagerError> {
    let mode = output.current_mode();
    let physical_width = if swaps_dimensions(output.transform()) {
        mode.height()
    } else {
        mode.width()
    };
    let logical_width = logical_extent(output, physical_width)?;
    i32::try_from(logical_width).map_err(|_| OutputManagerError::PositionOverflow)
}

fn swaps_dimensions(transform: smithay::utils::Transform) -> bool {
    matches!(
        transform,
        smithay::utils::Transform::_90
            | smithay::utils::Transform::_270
            | smithay::utils::Transform::Flipped90
            | smithay::utils::Transform::Flipped270
    )
}

fn logical_extent(output: &DisplayOutput, physical: u32) -> Result<u32, OutputManagerError> {
    let logical = if let Some(scale) = output.fractional_scale() {
        scale.physical_to_logical(i64::from(physical))
    } else {
        let scale = u64::from(output.scale());
        if scale == 0 {
            return Err(OutputManagerError::PositionOverflow);
        }

        let physical = u64::from(physical);
        i64::try_from(
            physical
                .checked_add(scale / 2)
                .ok_or(OutputManagerError::PositionOverflow)?
                / scale,
        )
        .ok()
    }
    .ok_or(OutputManagerError::PositionOverflow)?;

    u32::try_from(logical)
        .ok()
        .filter(|extent| *extent > 0)
        .ok_or(OutputManagerError::PositionOverflow)
}

#[cfg(test)]
#[path = "manager_tests.rs"]
mod tests;
