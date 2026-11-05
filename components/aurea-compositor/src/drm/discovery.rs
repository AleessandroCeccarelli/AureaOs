
 // SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli
//! Discovers connected DRM connectors and converts them into output descriptors.

use std::{error::Error, fmt};

use smithay::reexports::drm::control::{
    connector,
    Device as DrmControlDevice,
    ModeTypeFlags,
};

use crate::drm::{
    device::DrmDevice,
    output::{DrmModeInfo, DrmOutputDescriptor},
};
use crate::output::OutputId;

/// Errors produced while discovering DRM outputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrmDiscoveryError {
    /// The DRM device could not provide its resource handles.
    ResourceQueryFailed(String),

    /// The DRM device could not provide connector information.
    ConnectorQueryFailed {
        connector_id: u32,
        message: String,
    },

    /// Output identifiers overflowed while assigning discovered outputs.
    OutputIdOverflow,
}

impl fmt::Display for DrmDiscoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ResourceQueryFailed(message) => {
                write!(formatter, "Failed to query DRM resources: {message}")
            }
            Self::ConnectorQueryFailed {
                connector_id,
                message,
            } => {
                write!(
                    formatter,
                    "Failed to query DRM connector {connector_id}: {message}"
                )
            }
            Self::OutputIdOverflow => {
                formatter.write_str("DRM output identifier overflow")
            }
        }
    }
}

impl Error for DrmDiscoveryError {}

/// Discovers connected DRM outputs.
///
/// `first_output_id` is the ID assigned to the first connected output.
/// Subsequent IDs are assigned in the order returned by DRM.
///
/// Disconnected connectors and connectors without display modes are skipped.
/// The selected initial mode is the preferred mode, or the first valid mode
/// when no preferred mode is available.
pub fn discover_connected_outputs(
    device: &DrmDevice,
    first_output_id: OutputId,
) -> Result<Vec<DrmOutputDescriptor>, DrmDiscoveryError> {
    let resources = device
        .fd()
        .resource_handles()
        .map_err(|error| {
            DrmDiscoveryError::ResourceQueryFailed(error.to_string())
        })?;

    let mut descriptors = Vec::new();
    let mut next_output_id = first_output_id;

    for connector_handle in resources.connectors() {
        let connector_info = device
            .fd()
            .get_connector(*connector_handle, true)
            .map_err(|error| {
                DrmDiscoveryError::ConnectorQueryFailed {
                    connector_id: u32::from(*connector_handle),
                    message: error.to_string(),
                }
            })?;

        if connector_info.state() != connector::State::Connected {
            continue;
        }

        let drm_modes = connector_info.modes();

        if drm_modes.is_empty() {
            continue;
        }

        let modes = drm_modes
            .iter()
            .map(|mode| {
                let (width, height) = mode.size();

                DrmModeInfo::new(
                    u32::from(width),
                    u32::from(height),
                    u32::from(mode.vrefresh()).saturating_mul(1_000),
                )
            })
            .collect::<Vec<_>>();

        let preferred_index = drm_modes
            .iter()
            .position(|mode| {
                mode.mode_type().contains(ModeTypeFlags::PREFERRED)
            })
            .unwrap_or(0);

        let (current_width, current_height) =
            drm_modes[preferred_index].size();

        let current_mode = DrmModeInfo::new(
            u32::from(current_width),
            u32::from(current_height),
            u32::from(drm_modes[preferred_index].vrefresh())
                .saturating_mul(1_000),
        );

        let name = format!(
            "{}-{}",
            connector_info.interface().as_str(),
            connector_info.interface_id()
        );

        descriptors.push(DrmOutputDescriptor::new(
            next_output_id,
            name,
            modes,
            current_mode,
            1,
        ));

        next_output_id = next_output_id
            .checked_add(1)
            .ok_or(DrmDiscoveryError::OutputIdOverflow)?;
    }

    Ok(descriptors)
}
