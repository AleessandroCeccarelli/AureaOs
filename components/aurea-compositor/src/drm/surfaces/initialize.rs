// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Discovers connected DRM outputs and initializes their native KMS surfaces.

use std::collections::HashSet;

use smithay::reexports::drm::control::{Device as DrmControlDevice, connector};

use crate::drm::{
    cpu_presentation_backend::CpuPresentationBackend,
    device::DrmDevice,
    discovery::discover_connected_outputs,
    gbm_framebuffer_pool::GbmCpuFramebufferPool,
    output::DrmOutputDescriptor,
    presentation_backend::DrmPresentationBackend,
    surfaces::{
        DrmSurfaceInitFailure, DrmSurfaceManager, DrmSurfaceManagerError, ManagedDrmSurface,
    },
};

use crate::vulkan::VulkanDrmFramePool;

impl DrmSurfaceManager {
    /// Discovers outputs and creates one KMS surface for each usable connector.
    ///
    /// Each connector is checked against its encoder's possible CRTC mask.
    /// The selected CRTC is also checked against the manager's existing
    /// assignments before Smithay is asked to create the surface.
    pub fn initialize(device: &mut DrmDevice) -> Result<Self, DrmSurfaceInitFailure> {
        let descriptors = match discover_connected_outputs(device) {
            Ok(descriptors) => descriptors,
            Err(error) => {
                return Err(DrmSurfaceInitFailure {
                    partial_manager: Self::new(),
                    cause: DrmSurfaceManagerError::Discovery(error.to_string()),
                });
            }
        };
        Self::from_descriptors(device, &descriptors)
    }

    pub fn from_descriptors(
        device: &mut DrmDevice,
        descriptors: &[DrmOutputDescriptor],
    ) -> Result<Self, DrmSurfaceInitFailure> {
        let mut manager = Self::new();

        macro_rules! fail {
            ($cause:expr) => {{
                return Err(DrmSurfaceInitFailure {
                    partial_manager: manager,
                    cause: $cause,
                });
            }};
        }

        let resources = match device.fd().resource_handles() {
            Ok(resources) => resources,
            Err(error) => fail!(DrmSurfaceManagerError::Discovery(error.to_string())),
        };
        let node = match device.node() {
            Ok(node) => node,
            Err(error) => fail!(DrmSurfaceManagerError::Discovery(error.to_string())),
        };
        let available_crtcs = device.device().crtcs().to_vec();
        let mut reserved = HashSet::new();

        for descriptor in descriptors {
            let output_id = descriptor.id();
            let connector_handle = match descriptor.connector_handle() {
                Some(handle) => handle,
                None => fail!(DrmSurfaceManagerError::MissingNativeConfiguration(
                    output_id
                )),
            };
            let mode = match descriptor.native_mode() {
                Some(mode) => mode,
                None => fail!(DrmSurfaceManagerError::MissingNativeConfiguration(
                    output_id
                )),
            };

            let connector_info = match device.fd().get_connector(connector_handle, true) {
                Ok(info) => info,
                Err(error) => fail!(DrmSurfaceManagerError::ConnectorQuery {
                    output_id,
                    message: error.to_string(),
                }),
            };

            if connector_info.state() != connector::State::Connected {
                continue;
            }

            let mut selected_crtc = None;
            for encoder_handle in connector_info.encoders() {
                let encoder = match device.fd().get_encoder(*encoder_handle) {
                    Ok(encoder) => encoder,
                    Err(error) => fail!(DrmSurfaceManagerError::EncoderQuery {
                        output_id,
                        encoder_id: u32::from(*encoder_handle),
                        message: error.to_string(),
                    }),
                };
                let compatible = resources.filter_crtcs(encoder.possible_crtcs());
                selected_crtc = available_crtcs
                    .iter()
                    .copied()
                    .find(|crtc| compatible.contains(crtc) && !reserved.contains(crtc));
                if selected_crtc.is_some() {
                    break;
                }
            }

            let crtc = match selected_crtc {
                Some(crtc) => crtc,
                None => fail!(DrmSurfaceManagerError::NoCompatibleCrtc(output_id)),
            };
            if !reserved.insert(crtc) {
                fail!(DrmSurfaceManagerError::DuplicateCrtc(crtc));
            }

            let surface = match device
                .device_mut()
                .create_surface(crtc, mode, &[connector_handle])
            {
                Ok(surface) => surface,
                Err(error) => fail!(DrmSurfaceManagerError::SurfaceCreation {
                    output_id,
                    message: error.to_string(),
                }),
            };

            let presentation = DrmPresentationBackend::new(node, &surface);
            let cpu_presentation = CpuPresentationBackend::new(node, &surface);
            manager.crtc_owners.insert(crtc, output_id);
            manager.surfaces.insert(
                output_id,
                ManagedDrmSurface {
                    output_id,
                    connector: connector_handle,
                    native_modes: descriptor.native_modes().to_vec(),
                    surface,
                    presentation,
                    frame_pool: VulkanDrmFramePool::new(),
                    cpu_frame_pool: GbmCpuFramebufferPool::new(),
                    cpu_presentation,
                    direct_scanout_disabled: false,
                    staged_previous_mode: None,
                },
            );
        }

        Ok(manager)
    }
}
