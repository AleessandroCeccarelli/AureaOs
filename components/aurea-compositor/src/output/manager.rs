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

    /// Enables or disables an output.
    pub fn set_enabled(
        &mut self,
        output_id: OutputId,
        enabled: bool,
    ) -> Result<(), OutputManagerError> {
        let output = self
            .registry
            .get(output_id)
            .ok_or(OutputManagerError::UnknownOutput(output_id))?;
        if enabled && !output.is_enabled() {
            self.ensure_active_capacity(1)?;
        }
        self.registry
            .get_mut(output_id)
            .ok_or(OutputManagerError::UnknownOutput(output_id))?
            .set_enabled(enabled);
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
    pub fn set_scale(&mut self, output_id: OutputId, scale: u32) -> Result<(), OutputManagerError> {
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

    /// Changes the scale of an output to an exact fractional value.
    pub fn set_fractional_scale(
        &mut self,
        output_id: OutputId,
        scale: super::scale::ScaleFactor,
    ) -> Result<(), OutputManagerError> {
        let output = self
            .registry
            .get_mut(output_id)
            .ok_or(OutputManagerError::UnknownOutput(output_id))?;
        output.set_fractional_scale(scale);
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
mod tests {
    use super::*;

    #[test]
    fn logical_bounds_follow_position_and_integer_scale() {
        let mode = OutputMode::new(1920, 1080, 60_000).unwrap();
        let output = DisplayOutput::new(7, "DP-1", vec![mode], mode, 2).unwrap();
        let mut manager = OutputManager::new();
        manager.register(output, (-1920, 40)).unwrap();

        let bounds = manager.logical_bounds(7).unwrap().unwrap();
        assert_eq!(bounds.origin(), LogicalPoint { x: -1920, y: 40 });
        assert_eq!(
            bounds.size(),
            LogicalSize {
                width: 960,
                height: 540
            }
        );
    }

    #[test]
    fn logical_bounds_use_fractional_scale() {
        let mode = OutputMode::new(2560, 1440, 60_000).unwrap();
        let mut output = DisplayOutput::new(8, "DP-2", vec![mode], mode, 1).unwrap();
        output.set_fractional_scale(super::super::scale::ScaleFactor::from_percent(150).unwrap());
        let mut manager = OutputManager::new();
        manager.register(output, (100, -20)).unwrap();

        let bounds = manager.logical_bounds(8).unwrap().unwrap();
        assert_eq!(bounds.origin(), LogicalPoint { x: 100, y: -20 });
        assert_eq!(
            bounds.size(),
            LogicalSize {
                width: 1707,
                height: 960
            }
        );
    }

    #[test]
    fn logical_bounds_swap_dimensions_for_quarter_turn_outputs() {
        let mode = OutputMode::new(1920, 1080, 60_000).unwrap();
        let mut output = DisplayOutput::new(10, "DP-3", vec![mode], mode, 1).unwrap();
        output.set_transform(smithay::utils::Transform::_90);

        let mut manager = OutputManager::new();
        manager.register(output, (40, 70)).unwrap();

        let bounds = manager.logical_bounds(10).unwrap().unwrap();
        assert_eq!(bounds.origin(), LogicalPoint { x: 40, y: 70 });
        assert_eq!(
            bounds.size(),
            LogicalSize {
                width: 1080,
                height: 1920,
            }
        );
    }

    #[test]
    fn disabled_output_has_no_active_bounds() {
        let mode = OutputMode::new(1280, 720, 60_000).unwrap();
        let mut output = DisplayOutput::new(9, "HDMI-A-1", vec![mode], mode, 1).unwrap();
        output.set_enabled(false);
        let mut manager = OutputManager::new();
        manager.register(output, (0, 0)).unwrap();

        assert_eq!(manager.logical_bounds(9).unwrap(), None);
    }

    #[test]
    fn unknown_output_returns_an_error() {
        let manager = OutputManager::new();
        assert_eq!(
            manager.logical_bounds(404).unwrap_err(),
            OutputManagerError::UnknownOutput(404)
        );
    }
}
