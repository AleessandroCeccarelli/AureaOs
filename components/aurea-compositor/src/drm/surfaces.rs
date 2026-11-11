// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Owns native Smithay DRM surfaces for discovered physical outputs.

use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt,
};

use smithay::{
    backend::drm::{DrmNode, DrmSurface},
    reexports::drm::control::{Device as DrmControlDevice, connector, crtc},
};

use crate::{
    drm::{
        device::DrmDevice,
        discovery::discover_connected_outputs,
        output::DrmOutputDescriptor,
        presentation_backend::{DrmPresentationBackend, DrmPresentationBackendError},
    },
    output::OutputId,
};

/// Failures encountered while assigning CRTCs and creating DRM surfaces.
#[derive(Debug)]
pub enum DrmSurfaceManagerError {
    Discovery(String),
    MissingNativeConfiguration(OutputId),
    ConnectorQuery {
        output_id: OutputId,
        message: String,
    },
    EncoderQuery {
        output_id: OutputId,
        encoder_id: u32,
        message: String,
    },
    NoCompatibleCrtc(OutputId),
    DuplicateCrtc(crtc::Handle),
    SurfaceCreation {
        output_id: OutputId,
        message: String,
    },
}

impl fmt::Display for DrmSurfaceManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Discovery(message) => write!(f, "DRM output discovery failed: {message}"),
            Self::MissingNativeConfiguration(id) => write!(
                f,
                "DRM output {id} has no native connector/mode configuration"
            ),
            Self::ConnectorQuery { output_id, message } => write!(
                f,
                "Failed to query connector for output {output_id}: {message}"
            ),
            Self::EncoderQuery {
                output_id,
                encoder_id,
                message,
            } => write!(
                f,
                "Failed to query encoder {encoder_id} for output {output_id}: {message}"
            ),
            Self::NoCompatibleCrtc(id) => {
                write!(f, "No unused compatible CRTC is available for output {id}")
            }
            Self::DuplicateCrtc(crtc) => write!(f, "CRTC {crtc:?} was assigned more than once"),
            Self::SurfaceCreation { output_id, message } => write!(
                f,
                "Failed to create DRM surface for output {output_id}: {message}"
            ),
        }
    }
}

impl Error for DrmSurfaceManagerError {}

/// A physical output and its native KMS surface.
pub struct ManagedDrmSurface {
    output_id: OutputId,
    connector: connector::Handle,
    surface: DrmSurface,
    presentation: DrmPresentationBackend,
}

impl ManagedDrmSurface {
    pub fn output_id(&self) -> OutputId {
        self.output_id
    }
    pub fn connector(&self) -> connector::Handle {
        self.connector
    }
    pub fn crtc(&self) -> crtc::Handle {
        self.surface.crtc()
    }
    pub fn surface(&self) -> &DrmSurface {
        &self.surface
    }
    pub fn presentation(&self) -> &DrmPresentationBackend {
        &self.presentation
    }
    pub fn presentation_mut(&mut self) -> &mut DrmPresentationBackend {
        &mut self.presentation
    }
}

/// Owns all KMS surfaces created for one DRM device.
///
/// CRTC allocation is exclusive within this manager. A manager must be
/// discarded before its parent DRM device is removed or reset.
#[derive(Default)]
pub struct DrmSurfaceManager {
    surfaces: HashMap<OutputId, ManagedDrmSurface>,
    crtc_owners: HashMap<crtc::Handle, OutputId>,
}

impl DrmSurfaceManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, output_id: OutputId) -> Option<&ManagedDrmSurface> {
        self.surfaces.get(&output_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&OutputId, &ManagedDrmSurface)> {
        self.surfaces.iter()
    }

    pub fn crtc_owner(&self, crtc: crtc::Handle) -> Option<OutputId> {
        self.crtc_owners.get(&crtc).copied()
    }

    /// Discovers outputs and creates one KMS surface for each usable connector.
    ///
    /// Each connector is checked against its encoder's possible CRTC mask.
    /// The selected CRTC is also checked against the manager's existing
    /// assignments before Smithay is asked to create the surface.
    pub fn initialize(device: &mut DrmDevice) -> Result<Self, DrmSurfaceManagerError> {
        let descriptors = discover_connected_outputs(device)
            .map_err(|error| DrmSurfaceManagerError::Discovery(error.to_string()))?;
        Self::from_descriptors(device, &descriptors)
    }

    pub fn from_descriptors(
        device: &mut DrmDevice,
        descriptors: &[DrmOutputDescriptor],
    ) -> Result<Self, DrmSurfaceManagerError> {
        let mut manager = Self::new();
        let resources = device
            .fd()
            .resource_handles()
            .map_err(|error| DrmSurfaceManagerError::Discovery(error.to_string()))?;
        let available_crtcs = device.device().crtcs().to_vec();
        let mut reserved = HashSet::new();

        for descriptor in descriptors {
            let output_id = descriptor.id();
            let connector_handle = descriptor.connector_handle().ok_or(
                DrmSurfaceManagerError::MissingNativeConfiguration(output_id),
            )?;
            let mode = descriptor.native_mode().ok_or(
                DrmSurfaceManagerError::MissingNativeConfiguration(output_id),
            )?;

            let connector_info =
                device
                    .fd()
                    .get_connector(connector_handle, true)
                    .map_err(|error| DrmSurfaceManagerError::ConnectorQuery {
                        output_id,
                        message: error.to_string(),
                    })?;

            if connector_info.state() != connector::State::Connected {
                continue;
            }

            let mut selected_crtc = None;
            for encoder_handle in connector_info.encoders() {
                let encoder = device.fd().get_encoder(*encoder_handle).map_err(|error| {
                    DrmSurfaceManagerError::EncoderQuery {
                        output_id,
                        encoder_id: u32::from(*encoder_handle),
                        message: error.to_string(),
                    }
                })?;
                let compatible = resources.filter_crtcs(encoder.possible_crtcs());
                selected_crtc = available_crtcs
                    .iter()
                    .copied()
                    .find(|crtc| compatible.contains(crtc) && !reserved.contains(crtc));
                if selected_crtc.is_some() {
                    break;
                }
            }

            let crtc = selected_crtc.ok_or(DrmSurfaceManagerError::NoCompatibleCrtc(output_id))?;
            if !reserved.insert(crtc) {
                return Err(DrmSurfaceManagerError::DuplicateCrtc(crtc));
            }

            let surface = device
                .device_mut()
                .create_surface(crtc, mode, &[connector_handle])
                .map_err(|error| DrmSurfaceManagerError::SurfaceCreation {
                    output_id,
                    message: error.to_string(),
                })?;

            let presentation = DrmPresentationBackend::new(
                device
                    .node()
                    .map_err(|error| DrmSurfaceManagerError::Discovery(error.to_string()))?,
                &surface,
            );
            manager.crtc_owners.insert(crtc, output_id);
            manager.surfaces.insert(
                output_id,
                ManagedDrmSurface {
                    output_id,
                    connector: connector_handle,
                    surface,
                    presentation,
                },
            );
        }

        Ok(manager)
    }

    /// Routes a VBlank event to the presentation lifecycle for its CRTC.
    pub fn on_vblank(
        &mut self,
        node: DrmNode,
        crtc: crtc::Handle,
    ) -> Result<Option<crate::drm::presentation::DrmFramebufferId>, DrmPresentationBackendError>
    {
        let Some(output_id) = self.crtc_owners.get(&crtc).copied() else {
            return Ok(None);
        };
        let Some(output) = self.surfaces.get_mut(&output_id) else {
            return Ok(None);
        };
        if output.presentation().pending().is_none() {
            return Ok(None);
        }
        output.presentation_mut().on_vblank(node, crtc)
    }

    /// Drops all surfaces before their owning DRM device is torn down.
    pub fn clear(&mut self) {
        self.surfaces.clear();
        self.crtc_owners.clear();
    }
}
