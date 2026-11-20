// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Output configuration request types and transactional setters.

use super::OutputManager;
use super::super::{
    layout::OutputPosition,
    limits::MAX_ACTIVE_OUTPUTS,
    scale::ScaleFactor,
};
use crate::output::{OutputId, OutputMode};

/// Scale change requested as part of an output configuration transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputScale {
    Integer(u32),
    Fractional(ScaleFactor),
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

use super::OutputManagerError;

impl OutputManager {
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
            if !staged.layout.set_position(
                output_id,
                OutputPosition::new(position.0, position.1),
            ) {
                return Err(OutputManagerError::UnknownOutput(output_id));
            }
        }

        // Registry and layout are a single invariant even for disabled outputs,
        // whose logical bounds are intentionally absent.
        if !staged.layout.contains(output_id) {
            return Err(OutputManagerError::UnknownOutput(output_id));
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
        scale: ScaleFactor,
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
