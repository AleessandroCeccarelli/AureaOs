// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Runtime management of opened DRM devices.

use std::{collections::HashMap, error::Error, fmt};

use smithay::backend::drm::DrmNode;

use crate::drm::device::DrmDevice;

/// Errors returned by the DRM device manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrmDeviceManagerError {
    /// A DRM device with this identity is already open.
    DeviceAlreadyRegistered(DrmNode),
}

impl fmt::Display for DrmDeviceManagerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeviceAlreadyRegistered(node) => {
                write!(formatter, "DRM device is already open: {node}")
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
#[derive(Default)]
pub struct DrmDeviceManager {
    devices: HashMap<DrmNode, DrmDevice>,
}

impl DrmDeviceManager {
    /// Creates an empty DRM device manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an opened DRM device.
    ///
    /// A device identity can only be registered once. If the identity
    /// is already present, the new device is dropped and the existing
    /// device remains untouched.
    pub fn register(
        &mut self,
        node: DrmNode,
        device: DrmDevice,
    ) -> Result<(), DrmDeviceManagerError> {
        if self.devices.contains_key(&node) {
            return Err(DrmDeviceManagerError::DeviceAlreadyRegistered(node));
        }

        self.devices.insert(node, device);

        Ok(())
    }

    /// Returns an opened DRM device by its canonical identity.
    pub fn get(&self, node: DrmNode) -> Option<&DrmDevice> {
        self.devices.get(&node)
    }

    /// Returns a mutable opened DRM device by its canonical identity.
    pub fn get_mut(&mut self, node: DrmNode) -> Option<&mut DrmDevice> {
        self.devices.get_mut(&node)
    }

    /// Removes and returns an opened DRM device.
    ///
    /// Removing the device drops it from manager ownership. The caller
    /// receives the device and is responsible for any additional cleanup.
    pub fn remove(&mut self, node: DrmNode) -> Option<DrmDevice> {
        self.devices.remove(&node)
    }

    /// Returns whether a DRM device is currently open.
    pub fn contains(&self, node: DrmNode) -> bool {
        self.devices.contains_key(&node)
    }

    /// Returns an iterator over all opened DRM devices.
    pub fn iter(&self) -> impl Iterator<Item = (&DrmNode, &DrmDevice)> {
        self.devices.iter()
    }

    /// Returns a mutable iterator over all opened DRM devices.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&DrmNode, &mut DrmDevice)> {
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
mod tests {
    use super::*;

    #[test]
    fn manager_starts_empty() {
        let manager = DrmDeviceManager::new();

        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
        assert_eq!(manager.iter().count(), 0);
    }

    #[test]
    fn manager_can_be_created_with_default() {
        let manager = DrmDeviceManager::default();

        assert!(manager.is_empty());
    }
}
