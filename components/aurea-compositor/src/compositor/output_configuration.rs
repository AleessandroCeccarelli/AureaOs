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
    wayland::OutputPublicationError,
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
                write!(formatter, "Failed to publish output configuration: {source}")
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
    pub fn configure_output(
        &mut self,
        display: &DisplayHandle,
        output_id: OutputId,
        configuration: OutputConfiguration,
    ) -> Result<(), CompositorOutputConfigurationError> {
        let previous = self.output_manager.clone();

        self.output_manager
            .configure_output(output_id, configuration)
            .map_err(CompositorOutputConfigurationError::Manager)?;

        let changes = OutputReconciliation {
            updated: vec![output_id],
            ..OutputReconciliation::default()
        };

        self.publish_output_reconciliation(display, previous, &changes)
            .map_err(CompositorOutputConfigurationError::Publication)?;

        self.refresh_pointer_focus_at_current_location();
        self.request_all_output_frames();

        Ok(())
    }
}
