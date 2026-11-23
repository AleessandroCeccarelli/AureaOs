// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Discovers connected DRM connectors and converts them into output descriptors.

use std::{error::Error, fmt};

use smithay::reexports::drm::control::{Device as DrmControlDevice, ModeTypeFlags, connector};

use super::{connector_identity::DrmConnectorIdentity, edid::EdidInfo};

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
        }
    }
}

impl Error for DrmDiscoveryError {}

/// Derives a globally unique output ID from a DRM device and connector.
///
/// The upper 64 bits contain the DRM device identifier.
/// The lower 32 bits contain the connector identifier.
fn output_id_for_connector(device_id: u64, connector_id: u32) -> OutputId {
    (u128::from(device_id) << 32) | u128::from(connector_id)
}

/// Extracts the DRM device identifier from an output ID.
#[cfg(test)]
pub(super) fn drm_device_id_for_output(output_id: OutputId) -> u64 {
    (output_id >> 32) as u64
}

/// Reads and validates the connector's immutable EDID property.
///
/// Missing, malformed, or unreadable EDID is non-fatal: the output remains
/// usable, but must not receive an automatically inferred persistent identity.
fn read_connector_edid(device: &DrmDevice, connector: connector::Handle) -> Option<EdidInfo> {
    let properties = device.fd().get_properties(connector).ok()?;

    properties.iter().find_map(|(handle, raw_value)| {
        let property = device.fd().get_property(*handle).ok()?;
        if property.name().to_bytes() != b"EDID" {
            return None;
        }

        let blob_id = property.value_type().convert_value(*raw_value).as_blob()?;
        let bytes = device.fd().get_property_blob(blob_id).ok()?;
        EdidInfo::parse(&bytes)
    })
}

/// Discovers connected DRM outputs.
///
/// Each runtime output ID is derived from its DRM device identifier and
/// connector ID. IDs remain stable when connectors are enumerated in a
/// different order and remain unique across different DRM devices, but must
/// not be persisted as physical-monitor identities across device recreation.
///
/// Disconnected connectors and connectors without display modes are skipped.
/// The preferred mode is selected as the initial mode, falling back to the
/// first available mode when no preferred mode is reported.
pub fn discover_connected_outputs(
    device: &DrmDevice,
) -> Result<Vec<DrmOutputDescriptor>, DrmDiscoveryError> {
    // DRM resource queries use the underlying file descriptor directly.
    // They must not be performed while Smithay has paused the device.
    if !device.is_active() {
        return Err(DrmDiscoveryError::DeviceInactive);
    }

    let device_id = device.device().device_id() as u64;

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
        let output_id = output_id_for_connector(device_id, connector_id);

        let name = format!(
            "{}-{}",
            connector_info.interface().as_str(),
            connector_info.interface_id()
        );

        let edid = read_connector_edid(device, *connector_handle);
        let connector_name = format!(
            "{}-{}",
            connector_info.interface().as_str(),
            connector_info.interface_id()
        );
        let physical_identity = DrmConnectorIdentity::resolve(
            device.device_path(),
            &connector_name,
            std::path::Path::new("/sys"),
        );

        descriptors.push(
            DrmOutputDescriptor::new(output_id, name, modes, current_mode, 1)
                .with_drm_configuration(*connector_handle, drm_modes[preferred_index])
                .with_edid(edid)
                .with_physical_identity(physical_identity),
        );
    }

    Ok(descriptors)
}

#[cfg(test)]
#[path = "../../tests/unit/drm_discovery.rs"]
mod tests;
