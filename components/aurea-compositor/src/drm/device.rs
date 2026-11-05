// SPDX-License-Identifier: GPL-3.0-only
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli
//! DRM device discovery and access through an active libseat session.
use std::{
    error::Error,
    fmt,
    os::fd::OwnedFd,
    path::Path,
};
use smithay::backend::{
    drm::DrmDeviceFd,
    session::Session,
    udev,
};
use crate::drm::DrmSession;
/// Error returned while discovering or opening a DRM device.
#[derive(Debug)]
pub enum DrmDeviceError {
    /// No primary DRM device could be discovered.
    NoDevice,
    /// The libseat session is not active.
    SessionInactive,
    /// DRM device discovery failed.
    DiscoveryFailed(String),
    /// The DRM device could not be opened.
    OpenFailed(String),
}
impl fmt::Display for DrmDeviceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDevice => {
                formatter.write_str("No primary DRM device was found")
            }
            Self::SessionInactive => {
                formatter.write_str("Cannot open DRM device: session is inactive")
            }
            Self::DiscoveryFailed(message) => {
                write!(formatter, "Failed to discover DRM device: {message}")
            }
            Self::OpenFailed(message) => {
                write!(formatter, "Failed to open DRM device: {message}")
            }
        }
    }
}
impl Error for DrmDeviceError {}
/// Opens and owns a DRM device file descriptor.
pub struct DrmDevice {
    device_fd: DrmDeviceFd,
}
impl DrmDevice {
    /// Discovers and opens the primary GPU for the specified seat.
    pub fn open_primary(
        session: &DrmSession,
        seat: &str,
    ) -> Result<Self, DrmDeviceError> {
        let device_path = udev::primary_gpu(seat)
            .map_err(|error| {
                DrmDeviceError::DiscoveryFailed(error.to_string())
            })?
            .ok_or(DrmDeviceError::NoDevice)?;
        Self::open(session, &device_path)
    }
    /// Opens a DRM device using the active libseat session.
    pub fn open(
        session: &DrmSession,
        device_path: impl AsRef<Path>,
    ) -> Result<Self, DrmDeviceError> {
        if !session.session().is_active() {
            return Err(DrmDeviceError::SessionInactive);
        }
        let fd: OwnedFd = session
            .session()
            .open(device_path.as_ref())
            .map_err(|error| {
                DrmDeviceError::OpenFailed(error.to_string())
            })?;
        let device_fd = DrmDeviceFd::new(fd)
            .map_err(|error| {
                DrmDeviceError::OpenFailed(error.to_string())
            })?;
        Ok(Self { device_fd })
    }
    /// Returns a reference to the DRM device file descriptor.
    pub fn fd(&self) -> &DrmDeviceFd {
        &self.device_fd
    }
}
