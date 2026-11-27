// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Coordinates validated output configuration with Wayland publication.

use std::{error::Error, fmt};

use smithay::reexports::wayland_server::DisplayHandle;

use crate::output::{
    OutputId,
    manager::{OutputConfiguration, OutputManagerError, OutputReconciliation},
};

use super::{AureaCompositor, OutputReconciliationError};

/// Failure while applying an output configuration to compositor state.
#[derive(Debug)]
pub enum CompositorOutputConfigurationError {
    Manager(OutputManagerError),
    Publication(OutputReconciliationError),
}

impl fmt::Display for CompositorOutputConfigurationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Manager(source) => write!(formatter, "Invalid output configuration: {source}"),
            Self::Publication(source) => {
                write!(
                    formatter,
                    "Failed to publish output configuration: {source}"
                )
            }
        }
    }
}

impl Error for CompositorOutputConfigurationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Manager(source) => Some(source),
            Self::Publication(source) => Some(source),
        }
    }
}

impl AureaCompositor {
    /// Applies a validated output configuration and republishes its Wayland
    /// description. The output manager and Wayland global are rolled back if
    /// publication fails.
    ///
    /// This updates compositor and protocol state; the DRM/KMS modeset remains
    /// the responsibility of the active device backend.
    /// Validates a candidate output configuration without publishing it.
    ///
    /// The returned manager is detached from the live compositor state. DRM
    /// callers can retain it until the matching KMS page flip is confirmed.
    pub fn prepare_output_configuration(
        &self,
        output_id: OutputId,
        configuration: OutputConfiguration,
    ) -> Result<crate::output::manager::OutputManager, CompositorOutputConfigurationError> {
        self.output_manager
            .staged_configuration(output_id, configuration)
            .map_err(CompositorOutputConfigurationError::Manager)
    }

    /// Installs a previously validated candidate after its physical output
    /// change has completed. Publication failure restores the live manager.
    pub fn commit_prepared_output_configuration(
        &mut self,
        display: &DisplayHandle,
        output_id: OutputId,
        staged: crate::output::manager::OutputManager,
    ) -> Result<(), CompositorOutputConfigurationError> {
        let previous = std::mem::replace(&mut self.output_manager, staged);
        let changes = OutputReconciliation {
            updated: vec![output_id],
            ..OutputReconciliation::default()
        };

        if let Err(error) = self.publish_output_reconciliation(display, previous.clone(), &changes) {
            self.output_manager = previous;
            return Err(CompositorOutputConfigurationError::Publication(error));
        }

        self.refresh_pointer_focus_at_current_location();
        self.request_all_output_frames();
        Ok(())
    }

    /// Applies a validated output configuration and republishes its Wayland
    /// description. This convenience path is for logical-only changes.
    /// Physical mode changes must use prepare_output_configuration, complete
    /// KMS, then commit_prepared_output_configuration after VBlank.
    pub fn configure_output(
        &mut self,
        display: &DisplayHandle,
        output_id: OutputId,
        configuration: OutputConfiguration,
    ) -> Result<(), CompositorOutputConfigurationError> {
        let staged = self.prepare_output_configuration(output_id, configuration)?;
        self.commit_prepared_output_configuration(display, output_id, staged)
    }
}
