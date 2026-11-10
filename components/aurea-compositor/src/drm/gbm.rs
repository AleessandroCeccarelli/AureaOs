
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! GBM device ownership and lifecycle management.

use std::{collections::HashMap, error::Error, fmt};

use smithay::backend::{
    allocator::gbm::GbmDevice,
    drm::{DrmDeviceFd, DrmNode},
};

/// Errors returned by the GBM device manager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GbmDeviceManagerError {
    /// A GBM device is already registered for this DRM node.
    DeviceAlreadyRegistered(DrmNode),

    /// GBM initialization failed for the requested DRM node.
    InitializationFailed {
        node: DrmNode,
        message: String,
    },
}

impl fmt::Display for GbmDeviceManagerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeviceAlreadyRegistered(node) => {
                write!(formatter, "GBM device is already registered: {node}")
            }
            Self::InitializationFailed { node, message } => {
                write!(formatter, "Failed to initialize GBM device {node}: {message}")
            }
        }
    }
}

impl Error for GbmDeviceManagerError {}

/// Owns one GBM device for each initialized DRM device.
///
/// GBM devices share the reference-counted DRM file descriptor.
/// They do not independently open or acquire the DRM device.
pub struct GbmDeviceManager {
    devices: HashMap<DrmNode, GbmDevice<DrmDeviceFd>>,
}

impl Default for GbmDeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GbmDeviceManager {
    /// Creates an empty GBM device manager.
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }

    /// Initializes and registers a GBM device for an opened DRM node.
    ///
    /// The existing DRM descriptor is cloned so both managers retain
    /// shared ownership of the same underlying device.
    pub fn register(
        &mut self,
        node: DrmNode,
        drm_fd: &DrmDeviceFd,
    ) -> Result<(), GbmDeviceManagerError> {
        if self.devices.contains_key(&node) {
            return Err(GbmDeviceManagerError::DeviceAlreadyRegistered(node));
        }

        let device = GbmDevice::new(drm_fd.clone()).map_err(|error| {
            GbmDeviceManagerError::InitializationFailed {
                node,
                message: error.to_string(),
            }
        })?;

        self.devices.insert(node, device);

        Ok(())
    }

    /// Returns a GBM device for the requested DRM node.
    pub fn get(&self, node: DrmNode) -> Option<&GbmDevice<DrmDeviceFd>> {
        self.devices.get(&node)
    }

    /// Removes a GBM device and releases its GBM resources.
    pub fn remove(&mut self, node: DrmNode) -> bool {
        self.devices.remove(&node).is_some()
    }

    /// Removes every registered GBM device.
    pub fn clear(&mut self) {
        self.devices.clear();
    }

    /// Returns whether a GBM device is registered for this DRM node.
    pub fn contains(&self, node: DrmNode) -> bool {
        self.devices.contains_key(&node)
    }

    /// Returns the number of registered GBM devices.
    pub fn len(&self) -> usize {
        self.devices.len()
    }

    /// Returns whether no GBM devices are registered.
    pub fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }

    /// Returns an iterator over registered GBM devices.
    pub fn iter(&self) -> impl Iterator<Item = (&DrmNode, &GbmDevice<DrmDeviceFd>)> {
        self.devices.iter()
    }
}
