// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Initializes compositor outputs from the DRM backend.

use std::{error::Error, fmt};

use crate::{
    drm::{
        device::DrmDevice,
        discovery::{discover_connected_outputs, DrmDiscoveryError},
        output::{register_output, DrmOutputError},
    },
    output::{
        layout::OutputPosition,
        manager::OutputManager,
        OutputId,
    },
};

/// Errors produced while initializing DRM outputs.
#[derive(Debug)]
pub enum DrmOutputsError {
    Discovery(DrmDiscoveryError),
    Registration {
        output_id: OutputId,
        source: DrmOutputError,
    },
    PositionOverflow,
}

impl fmt::Display for DrmOutputsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Discovery(source) => {
                write!(formatter, "Failed to discover DRM outputs: {source}")
            }
            Self::Registration { output_id, source } => {
                write!(
                    formatter,
                    "Failed to register DRM output {output_id}: {source}"
                )
            }
            Self::PositionOverflow => {
                formatter.write_str("DRM output layout position overflow")
            }
        }
    }
}

impl Error for DrmOutputsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Discovery(source) => Some(source),
            Self::Registration { source, .. } => Some(source),
            Self::PositionOverflow => None,
        }
    }
}

/// Discovers connected DRM outputs and registers them in the output manager.
///
/// Outputs are placed horizontally from left to right, starting at `(0, 0)`.
/// The next output's x-position is calculated from the preceding output's
/// current mode width. `first_output_id` is the ID assigned to the first
/// discovered output.
pub fn initialize_drm_outputs(
    device: &DrmDevice,
    manager: &mut OutputManager,
    first_output_id: OutputId,
) -> Result<usize, DrmOutputsError> {
    let descriptors = discover_connected_outputs(device, first_output_id)
        .map_err(DrmOutputsError::Discovery)?;

    let mut next_x = 0_i32;
    let mut registered_count = 0;

    for descriptor in descriptors {
        let output_id = descriptor.id();
        let width = descriptor.current_mode().width();

        register_output(
            manager,
            descriptor,
            OutputPosition::new(next_x, 0),
        )
        .map_err(|source| DrmOutputsError::Registration {
            output_id,
            source,
        })?;

        let width = i32::try_from(width)
            .map_err(|_| DrmOutputsError::PositionOverflow)?;

        next_x = next_x
            .checked_add(width)
            .ok_or(DrmOutputsError::PositionOverflow)?;

        registered_count += 1;
    }

    Ok(registered_count)
}
