// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use super::{DisplayOutput, OutputId, OutputLayout, OutputMode, OutputPosition, OutputRegistry};
use std::fmt;

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

        // The registry and layout are updated only after duplicate validation.
        if !self.registry.register(output) {
            return Err(OutputManagerError::DuplicateOutput(id));
        }

        let inserted = self
            .layout
            .insert(id, OutputPosition::new(position.0, position.1));

        if !inserted {
            // Keep registry and layout consistent if insertion unexpectedly fails.
            self.registry.remove(id);
            return Err(OutputManagerError::DuplicateOutput(id));
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
mod tests {
    use super::*;

    fn sample_output() -> DisplayOutput {
        let mode = OutputMode::new(1920, 1080, 60_000).unwrap();

        DisplayOutput::new(1, "HDMI-A-1", vec![mode], mode, 1).unwrap()
    }

    #[test]
    fn manager_starts_empty() {
        let manager = OutputManager::new();
        assert!(manager.registry().is_empty());
        assert!(manager.layout().is_empty());
    }

    #[test]
    fn registers_output_and_position() {
        let mut manager = OutputManager::new();

        assert!(manager.register(sample_output(), (0, 0)).is_ok());
        assert_eq!(manager.registry().len(), 1);
        assert_eq!(
            manager.layout().get(1),
            Some(OutputPosition::new(0, 0))
        );
    }

    #[test]
    fn rejects_duplicate_output() {
        let mut manager = OutputManager::new();

        assert!(manager.register(sample_output(), (0, 0)).is_ok());
        assert_eq!(
            manager.register(sample_output(), (1920, 0)),
            Err(OutputManagerError::DuplicateOutput(1))
        );
        assert_eq!(manager.registry().len(), 1);
        assert_eq!(manager.layout().len(), 1);
    }

    #[test]
    fn unregister_removes_output_and_position() {
        let mut manager = OutputManager::new();
        manager.register(sample_output(), (0, 0)).unwrap();

        assert!(manager.unregister(1));
        assert!(manager.registry().is_empty());
        assert!(manager.layout().is_empty());
        assert!(!manager.unregister(1));
    }

    #[test]
    fn rejects_unsupported_mode_and_zero_scale() {
        let mut manager = OutputManager::new();
        manager.register(sample_output(), (0, 0)).unwrap();

        let unsupported = OutputMode::new(1280, 720, 60_000).unwrap();

        assert_eq!(
            manager.set_mode(1, unsupported),
            Err(OutputManagerError::InvalidMode {
                output: 1,
                mode: unsupported,
            })
        );

        assert_eq!(
            manager.set_scale(1, 0),
            Err(OutputManagerError::InvalidScale {
                output: 1,
                scale: 0,
            })
        );
    }
}
