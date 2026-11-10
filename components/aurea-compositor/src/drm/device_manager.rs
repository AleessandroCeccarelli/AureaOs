// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Runtime management of opened DRM devices.

use std::{collections::HashMap, error::Error, fmt, hash::Hash};

use smithay::backend::drm::DrmNode;

use crate::drm::device::{DrmDevice, DrmDeviceError};

/// The identity and session operations required of an owned DRM device.
///
/// The runtime implementation is [`DrmDevice`]. Keeping this small boundary
/// lets lifecycle coordination and ownership be tested without DRM hardware.
pub trait ManagedDrmDevice {
    /// Key type used by the manager. Production devices use `DrmNode`.
    type Identity: Copy + Eq + Hash + fmt::Display;
    /// Returns the canonical DRM identity.
    fn identity(&self) -> Result<Self::Identity, DrmDeviceManagerError>;
    /// Pauses device access for the current session.
    fn pause(&mut self);
    /// Reactivates the device after session activation.
    fn activate(&mut self) -> Result<(), DrmDeviceError>;
}

impl ManagedDrmDevice for DrmDevice {
    type Identity = DrmNode;

    fn identity(&self) -> Result<Self::Identity, DrmDeviceManagerError> {
        self.node()
            .map_err(|_| DrmDeviceManagerError::DeviceIdentityUnavailable)
    }

    fn pause(&mut self) {
        DrmDevice::pause(self);
    }

    fn activate(&mut self) -> Result<(), DrmDeviceError> {
        DrmDevice::activate(self)
    }
}

/// Errors returned by the DRM device manager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrmDeviceManagerError {
    /// A DRM device with this identity is already open.
    DeviceAlreadyRegistered(String),
    /// The opened device did not expose a canonical DRM identity.
    DeviceIdentityUnavailable,
}

impl fmt::Display for DrmDeviceManagerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeviceAlreadyRegistered(identity) => {
                write!(formatter, "DRM device is already open: {identity}")
            }
            Self::DeviceIdentityUnavailable => {
                formatter.write_str("Cannot identify opened DRM device")
            }
        }
    }
}

impl Error for DrmDeviceManagerError {}

/// Owns and manages opened DRM devices.
///
/// Devices are indexed by their canonical Smithay `DrmNode` identity.
/// This manager owns runtime device instances and their lifecycle.
///
/// Discovery metadata remains the responsibility of `DrmDeviceRegistry`.
pub struct DrmDeviceManager<D = DrmDevice, K = DrmNode> {
    devices: HashMap<K, D>,
}

impl<D, K> Default for DrmDeviceManager<D, K> {
    fn default() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }
}

impl<D, K> DrmDeviceManager<D, K>
where
    D: ManagedDrmDevice<Identity = K>,
    K: Copy + Eq + Hash + fmt::Display,
{
    /// Creates an empty DRM device manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an opened DRM device.
    ///
    /// A device identity can only be registered once. If the identity
    /// is already present, the new device is dropped and the existing
    /// device remains untouched.
    pub fn register(&mut self, device: D) -> Result<K, DrmDeviceManagerError> {
        let node = device.identity()?;
        if self.devices.contains_key(&node) {
            return Err(DrmDeviceManagerError::DeviceAlreadyRegistered(
                node.to_string(),
            ));
        }

        self.devices.insert(node, device);

        Ok(node)
    }

    /// Returns an opened DRM device by its canonical identity.
    pub fn get(&self, node: K) -> Option<&D> {
        self.devices.get(&node)
    }

    /// Returns a mutable opened DRM device by its canonical identity.
    pub fn get_mut(&mut self, node: K) -> Option<&mut D> {
        self.devices.get_mut(&node)
    }

    /// Removes and returns an opened DRM device.
    ///
    /// Removing the device drops it from manager ownership. The caller
    /// receives the device and is responsible for any additional cleanup.
    pub fn remove(&mut self, node: K) -> Option<D> {
        self.devices.remove(&node)
    }

    /// Returns whether a DRM device is currently open.
    pub fn contains(&self, node: K) -> bool {
        self.devices.contains_key(&node)
    }

    /// Returns an iterator over all opened DRM devices.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &D)> {
        self.devices.iter()
    }

    /// Returns a mutable iterator over all opened DRM devices.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut D)> {
        self.devices.iter_mut()
    }

    /// Returns the number of opened DRM devices.
    pub fn len(&self) -> usize {
        self.devices.len()
    }

    /// Returns whether no DRM devices are currently open.
    pub fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }
}

#[cfg(test)]
#[path = "../../tests/unit/drm_device_manager.rs"]
mod tests;
