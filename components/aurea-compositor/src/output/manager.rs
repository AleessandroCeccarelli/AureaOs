
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use super::{
    DisplayOutput,
    OutputId,
    OutputLayout,
    OutputMode,
    OutputPosition,
    OutputRegistry,
};
use std::{collections::BTreeSet, fmt};

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
        }
    }
}

impl std::error::Error for OutputManagerError {}

/// Coordinates output registration, layout, and configuration.
#[derive(Debug, Default)]
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

#[cfg(test)]
#[path = "../../tests/unit/output_manager.rs"]
mod tests;
