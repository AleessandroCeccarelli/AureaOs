// SPDX-License-Identifier: GPL-3.0-only
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! DRM device access through an active libseat session.

use std::os::fd::OwnedFd;

use smithay::backend::{
    drm::DrmDeviceFd,
    session::Session,
};

use crate::drm::DrmSession;

/// Error returned when opening the DRM device.
#[derive(Debug)]
pub enum DrmDeviceError {
    /// The libseat session is not active.
    SessionInactive,

    /// The DRM device could not be opened.
    OpenFailed(String),
}

impl std::fmt::Display for DrmDeviceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SessionInactive => {
                formatter.write_str("Cannot open DRM device: session is inactive")
            }
            Self::OpenFailed(message) => {
                write!(formatter, "Failed to open DRM device: {message}")
            }
        }
    }
}

impl std::error::Error for DrmDeviceError {}

/// Opens and owns a DRM device file descriptor.
pub struct DrmDevice {
    device_fd: DrmDeviceFd,
}

impl DrmDevice {
    /// Opens a DRM device using the active libseat session.
    pub fn open(
        session: &DrmSession,
        device_path: &str,
    ) -> Result<Self, DrmDeviceError> {
        if !session.session().is_active() {
            return Err(DrmDeviceError::SessionInactive);
        }

        let fd: OwnedFd = session
            .session()
            .open(device_path)
            .map_err(|error| DrmDeviceError::OpenFailed(error.to_string()))?;

        let device_fd = DrmDeviceFd::new(fd)
            .map_err(|error| DrmDeviceError::OpenFailed(error.to_string()))?;

        Ok(Self { device_fd })
    }

    /// Returns a reference to the DRM device file descriptor.
    pub fn fd(&self) -> &DrmDeviceFd {
        &self.device_fd
    }
}
