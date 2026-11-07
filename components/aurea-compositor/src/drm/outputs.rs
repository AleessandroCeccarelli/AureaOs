// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Initializes compositor outputs from the DRM backend.

use std::{collections::BTreeSet, error::Error, fmt};

use crate::{
    drm::{
        device::DrmDevice,
        discovery::{DrmDiscoveryError, discover_connected_outputs},
        output::DrmOutputError,
    },
    output::{
        OutputId,
        manager::{OutputManager, OutputManagerError},
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
    Manager(OutputManagerError),
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
                    "Failed to prepare DRM output {output_id}: {source}"
                )
            }
            Self::Manager(source) => {
                write!(
                    formatter,
                    "Output manager failed to register DRM outputs: {source}"
                )
            }
            Self::PositionOverflow => formatter.write_str("DRM output layout position overflow"),
        }
    }
}

impl Error for DrmOutputsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Discovery(source) => Some(source),
            Self::Registration { source, .. } => Some(source),
            Self::Manager(source) => Some(source),
            Self::PositionOverflow => None,
        }
    }
}

/// Discovers connected DRM outputs and registers them in the output manager.
///
/// Output IDs are derived from the DRM device and connector identities,
/// so this function does not need a caller-provided ID namespace.
///
/// Outputs are placed horizontally from left to right, starting at `(0, 0)`.
/// The next output's x-position is calculated from the preceding output's
/// current mode width.
///
/// All descriptors, IDs, and positions are validated before the output
/// manager is mutated. Registration is performed atomically as a batch.
pub fn initialize_drm_outputs(
    device: &DrmDevice,
    manager: &mut OutputManager,
) -> Result<usize, DrmOutputsError> {
    let descriptors = discover_connected_outputs(device).map_err(DrmOutputsError::Discovery)?;

    let mut staged_outputs = Vec::with_capacity(descriptors.len());
    let mut staged_ids = BTreeSet::new();
    let mut next_x = 0_i32;

    for descriptor in descriptors {
        let output_id = descriptor.id();

        if manager.registry().get(output_id).is_some() || !staged_ids.insert(output_id) {
            return Err(DrmOutputsError::Registration {
                output_id,
                source: DrmOutputError::DuplicateOutputId,
            });
        }

        let width = i32::try_from(descriptor.current_mode().width())
            .map_err(|_| DrmOutputsError::PositionOverflow)?;

        let output = descriptor
            .into_display_output()
            .map_err(|source| DrmOutputsError::Registration { output_id, source })?;

        staged_outputs.push((output, (next_x, 0)));

        next_x = next_x
            .checked_add(width)
            .ok_or(DrmOutputsError::PositionOverflow)?;
    }

    let output_count = staged_outputs.len();

    manager
        .register_batch(staged_outputs)
        .map_err(DrmOutputsError::Manager)?;

    Ok(output_count)
}
