// SPDX-License-Identifier: GPL-3.0-or-later

use super::{OutputLayout, OutputMode, OutputRegistry};
use std::fmt;

/// Errors returned by output management operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputManagerError {
    UnknownOutput(String),
    InvalidMode {
        output: String,
        mode: OutputMode,
    },
    InvalidScale {
        output: String,
        scale: u32,
    },
}

impl fmt::Display for OutputManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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
        output: super::DisplayOutput,
        position: (i32, i32),
    ) -> Result<(), OutputManagerError> {
        let id = output.id().to_owned();
        self.registry.register(output);
        self.layout.set_position(id, position);
        Ok(())
    }

    /// Unregisters an output and removes it from the layout.
    pub fn unregister(&mut self, output_id: &str) -> bool {
        self.layout.remove(output_id);
        self.registry.remove(output_id).is_some()
    }

    /// Enables or disables an output.
    pub fn set_enabled(
        &mut self,
        output_id: &str,
        enabled: bool,
    ) -> Result<(), OutputManagerError> {
        let output = self
            .registry
            .get_mut(output_id)
            .ok_or_else(|| OutputManagerError::UnknownOutput(output_id.to_owned()))?;

        output.set_enabled(enabled);
        Ok(())
    }

    /// Changes the active mode of an output.
    pub fn set_mode(
        &mut self,
        output_id: &str,
        mode: OutputMode,
    ) -> Result<(), OutputManagerError> {
        let output = self
            .registry
            .get_mut(output_id)
            .ok_or_else(|| OutputManagerError::UnknownOutput(output_id.to_owned()))?;

        if !output.supports_mode(&mode) {
            return Err(OutputManagerError::InvalidMode {
                output: output_id.to_owned(),
                mode,
            });
        }

        output.set_mode(mode);
        Ok(())
    }

    /// Changes the scale of an output.
    pub fn set_scale(
        &mut self,
        output_id: &str,
        scale: u32,
    ) -> Result<(), OutputManagerError> {
        if scale == 0 {
            return Err(OutputManagerError::InvalidScale {
                output: output_id.to_owned(),
                scale,
            });
        }

        let output = self
            .registry
            .get_mut(output_id)
            .ok_or_else(|| OutputManagerError::UnknownOutput(output_id.to_owned()))?;

        output.set_scale(scale);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manager_starts_empty() {
        let manager = OutputManager::new();
        assert_eq!(manager.registry().iter().count(), 0);
    }
}
