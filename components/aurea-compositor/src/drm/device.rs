// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! DRM device discovery, access and lifecycle through Smithay and libseat.

use smithay::backend::session::Session;

use std::{
    error::Error,
    fmt,
    os::fd::OwnedFd,
    path::{Path, PathBuf},
};

use smithay::backend::drm::DrmNode;
use smithay::{
    backend::{
        drm::{DrmDevice as SmithayDrmDevice, DrmDeviceFd, DrmDeviceNotifier},
        udev,
    },
    reexports::rustix::fs::OFlags,
    utils::DeviceFd,
};

use crate::drm::DrmSession;

/// Error returned while discovering or managing a DRM device.
#[derive(Debug)]
pub enum DrmDeviceError {
    /// No primary DRM device could be discovered.
    NoDevice,

    /// The libseat session is not active.
    SessionInactive,

    /// The specified device path is invalid.
    InvalidDevicePath(PathBuf),

    /// DRM device discovery failed.
    DiscoveryFailed(String),

    /// The DRM device could not be opened or initialized.
    OpenFailed(String),

    /// The DRM device could not be activated.
    ActivationFailed(String),

    /// The DRM notifier was already transferred.
    NotifierAlreadyTransferred,
}

impl fmt::Display for DrmDeviceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDevice => formatter.write_str("No primary DRM device was found"),
            Self::SessionInactive => {
                formatter.write_str("Cannot open DRM device: session is inactive")
            }
            Self::InvalidDevicePath(path) => {
                write!(formatter, "Invalid DRM device path: {}", path.display())
            }
            Self::DiscoveryFailed(message) => {
                write!(formatter, "Failed to discover DRM device: {message}")
            }
            Self::OpenFailed(message) => {
                write!(
                    formatter,
                    "Failed to open or initialize DRM device: {message}"
                )
            }
            Self::ActivationFailed(message) => {
                write!(formatter, "Failed to activate DRM device: {message}")
            }
            Self::NotifierAlreadyTransferred => {
                formatter.write_str("The DRM device notifier was already transferred")
            }
        }
    }
}

impl Error for DrmDeviceError {}

/// Owns a Smithay DRM device and its Calloop notifier.
pub struct DrmDevice {
    device: SmithayDrmDevice,
    notifier: Option<DrmDeviceNotifier>,
}

impl DrmDevice {
    /// Discovers and opens the primary GPU for the specified seat.
    pub fn open_primary(session: &mut DrmSession, seat: &str) -> Result<Self, DrmDeviceError> {
        let device_path = udev::primary_gpu(seat)
            .map_err(|error| DrmDeviceError::DiscoveryFailed(error.to_string()))?
            .ok_or(DrmDeviceError::NoDevice)?;

        Self::open(session, &device_path)
    }

    /// Opens and initializes a DRM device through the active libseat session.
    pub fn open(
        session: &mut DrmSession,
        device_path: impl AsRef<Path>,
    ) -> Result<Self, DrmDeviceError> {
        let device_path = device_path.as_ref();

        if !device_path.is_absolute() {
            return Err(DrmDeviceError::InvalidDevicePath(device_path.to_path_buf()));
        }

        if !session.is_active() {
            return Err(DrmDeviceError::SessionInactive);
        }

        let fd: OwnedFd = session
            .session()
            .open(
                device_path,
                OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY | OFlags::NONBLOCK,
            )
            .map_err(|error| DrmDeviceError::OpenFailed(error.to_string()))?;

        let device_fd = DrmDeviceFd::new(DeviceFd::from(fd));

        let (device, notifier) = SmithayDrmDevice::new(device_fd, true)
            .map_err(|error| DrmDeviceError::OpenFailed(error.to_string()))?;

        Ok(Self {
            device,
            notifier: Some(notifier),
        })
    }

    /// Returns a reference to the underlying Smithay DRM device.
    pub fn device(&self) -> &SmithayDrmDevice {
        &self.device
    }

    /// Returns Smithay's canonical identity for this opened DRM device.
    pub fn node(&self) -> Result<DrmNode, DrmDeviceError> {
        DrmNode::from_dev_id(self.device.device_id() as u64)
            .map_err(|error| DrmDeviceError::OpenFailed(error.to_string()))
    }

    /// Returns a mutable reference to the underlying Smithay DRM device.
    pub fn device_mut(&mut self) -> &mut SmithayDrmDevice {
        &mut self.device
    }

    /// Returns a reference to the DRM device file descriptor.
    pub fn fd(&self) -> &DrmDeviceFd {
        self.device.device_fd()
    }

    /// Transfers the DRM event notifier to the Calloop event loop.
    ///
    /// This can only be called once.
    pub fn take_notifier(&mut self) -> Result<DrmDeviceNotifier, DrmDeviceError> {
        self.notifier
            .take()
            .ok_or(DrmDeviceError::NotifierAlreadyTransferred)
    }

    /// Pauses the DRM device.
    pub fn pause(&mut self) {
        self.device.pause();
    }

    /// Reactivates the DRM device.
    ///
    /// Connectors are reset when the device is reactivated.
    pub fn activate(&mut self) -> Result<(), DrmDeviceError> {
        self.device
            .activate(true)
            .map_err(|error| DrmDeviceError::ActivationFailed(error.to_string()))
    }

    /// Returns whether the DRM device is active.
    pub fn is_active(&self) -> bool {
        self.device.is_active()
    }
}
