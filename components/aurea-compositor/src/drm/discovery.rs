// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Discovers connected DRM connectors and converts them into output descriptors.

use std::{error::Error, fmt};

use smithay::reexports::drm::control::{Device as DrmControlDevice, ModeTypeFlags, connector};

use crate::drm::{
    device::DrmDevice,
    output::{DrmModeInfo, DrmOutputDescriptor},
};
use crate::output::OutputId;

/// Errors produced while discovering DRM outputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrmDiscoveryError {
    /// The DRM device is currently inactive.
    DeviceInactive,

    /// The DRM device could not provide its resource handles.
    ResourceQueryFailed(String),

    /// The DRM device could not provide connector information.
    ConnectorQueryFailed { connector_id: u32, message: String },

    /// An output identifier could not be represented.
    OutputIdOverflow,
}

impl fmt::Display for DrmDiscoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeviceInactive => {
                formatter.write_str("Cannot discover DRM outputs while the device is inactive")
            }
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
            Self::OutputIdOverflow => formatter.write_str("DRM output identifier overflow"),
        }
    }
}

impl Error for DrmDiscoveryError {}

/// Derives a stable output ID from a DRM connector ID.
///
/// `first_output_id` acts as the base namespace for this DRM device.
/// The resulting ID does not depend on connector enumeration order.
fn output_id_for_connector(
    first_output_id: OutputId,
    connector_id: u32,
) -> Result<OutputId, DrmDiscoveryError> {
    first_output_id
        .checked_add(u64::from(connector_id))
        .ok_or(DrmDiscoveryError::OutputIdOverflow)
}

/// Discovers connected DRM outputs.
///
/// Each output ID is derived from its DRM connector ID and the supplied
/// device namespace. IDs therefore remain stable when other connectors
/// are connected, disconnected, or returned in a different order.
///
/// Disconnected connectors and connectors without display modes are skipped.
/// The selected initial mode is the preferred mode, or the first available
/// mode when no preferred mode is available.
pub fn discover_connected_outputs(
    device: &DrmDevice,
    first_output_id: OutputId,
) -> Result<Vec<DrmOutputDescriptor>, DrmDiscoveryError> {
    // drm-rs operations use the underlying file descriptor directly.
    // They must not be performed while Smithay has paused the device.
    if !device.is_active() {
        return Err(DrmDiscoveryError::DeviceInactive);
    }

    let resources = device
        .fd()
        .resource_handles()
        .map_err(|error| DrmDiscoveryError::ResourceQueryFailed(error.to_string()))?;

    let mut descriptors = Vec::new();

    for connector_handle in resources.connectors() {
        let connector_info =
            device
                .fd()
                .get_connector(*connector_handle, true)
                .map_err(|error| DrmDiscoveryError::ConnectorQueryFailed {
                    connector_id: u32::from(*connector_handle),
                    message: error.to_string(),
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
            .position(|mode| mode.mode_type().contains(ModeTypeFlags::PREFERRED))
            .unwrap_or(0);

        let (current_width, current_height) = drm_modes[preferred_index].size();

        let current_mode = DrmModeInfo::new(
            u32::from(current_width),
            u32::from(current_height),
            u32::from(drm_modes[preferred_index].vrefresh()).saturating_mul(1_000),
        );

        let connector_id = u32::from(*connector_handle);
        let output_id = output_id_for_connector(first_output_id, connector_id)?;

        let name = format!(
            "{}-{}",
            connector_info.interface().as_str(),
            connector_info.interface_id()
        );

        descriptors.push(DrmOutputDescriptor::new(
            output_id,
            name,
            modes,
            current_mode,
            1,
        ));
    }

    Ok(descriptors)
}

#[cfg(test)]
#[path = "../../tests/unit/drm_discovery.rs"]
mod tests;
