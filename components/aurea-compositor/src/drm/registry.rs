// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Registry of discovered DRM devices.
//!
//! The registry uses Smithay's `DrmNode` as the canonical device identity.
//! It tracks device paths and the compositor's primary DRM device.
//! Opening devices and managing their runtime lifecycle are handled elsewhere.

use std::{
    collections::HashMap,
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

use smithay::backend::drm::{DrmNode, NodeType};

/// Metadata associated with a discovered DRM device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrmDeviceInfo {
    node: DrmNode,
    path: PathBuf,
    render_node_path: Option<PathBuf>,
}

impl DrmDeviceInfo {
    /// Returns the canonical DRM device identity.
    pub fn node(&self) -> DrmNode {
        self.node
    }

    /// Returns the primary DRM device path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the matching render node path, when available.
    pub fn render_node_path(&self) -> Option<&Path> {
        self.render_node_path.as_deref()
    }

    /// Returns whether this device exposes a render node.
    pub fn has_render_node(&self) -> bool {
        self.render_node_path.is_some()
    }
}

/// Errors returned by the DRM device registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrmDeviceRegistryError {
    /// The supplied DRM device path is not absolute.
    InvalidDevicePath(PathBuf),

    /// The DRM node is already registered.
    DeviceAlreadyRegistered(DrmNode),

    /// The requested DRM node is not registered.
    UnknownDevice(DrmNode),
}

impl fmt::Display for DrmDeviceRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDevicePath(path) => {
                write!(formatter, "Invalid DRM device path: {}", path.display())
            }
            Self::DeviceAlreadyRegistered(node) => {
                write!(formatter, "DRM device is already registered: {node}")
            }
            Self::UnknownDevice(node) => {
                write!(formatter, "Unknown DRM device: {node}")
            }
        }
    }
}

impl Error for DrmDeviceRegistryError {}

/// Tracks discovered DRM devices by their canonical Smithay identity.
///
/// This registry owns inventory metadata only. It does not open device
/// descriptors, create DRM devices, or manage Calloop notifiers.
#[derive(Debug, Default)]
pub struct DrmDeviceRegistry {
    devices: HashMap<DrmNode, DrmDeviceInfo>,
    primary_device: Option<DrmNode>,
}

impl DrmDeviceRegistry {
    /// Creates an empty DRM device registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a discovered DRM device.
    ///
    /// The supplied path must be absolute. The matching render-node path
    /// is discovered through Smithay when one is available.
    pub fn register(
        &mut self,
        node: DrmNode,
        path: impl Into<PathBuf>,
    ) -> Result<(), DrmDeviceRegistryError> {
        let path = path.into();

        if !path.is_absolute() {
            return Err(DrmDeviceRegistryError::InvalidDevicePath(path));
        }

        if self.devices.contains_key(&node) {
            return Err(DrmDeviceRegistryError::DeviceAlreadyRegistered(node));
        }

        let render_node_path = node.dev_path_with_type(NodeType::Render);

        let info = DrmDeviceInfo {
            node,
            path,
            render_node_path,
        };

        self.devices.insert(node, info);

        Ok(())
    }

    /// Removes a DRM device from the registry.
    ///
    /// If the removed device was primary, the primary selection is cleared.
    /// Returns the removed metadata when the device was registered.
    pub fn remove(&mut self, node: DrmNode) -> Option<DrmDeviceInfo> {
        let removed = self.devices.remove(&node)?;

        if self.primary_device == Some(node) {
            self.primary_device = None;
        }

        Some(removed)
    }

    /// Returns metadata for a registered DRM device.
    pub fn get(&self, node: DrmNode) -> Option<&DrmDeviceInfo> {
        self.devices.get(&node)
    }

    /// Sets the primary DRM device.
    ///
    /// The device must already be registered.
    pub fn set_primary(&mut self, node: DrmNode) -> Result<(), DrmDeviceRegistryError> {
        if !self.devices.contains_key(&node) {
            return Err(DrmDeviceRegistryError::UnknownDevice(node));
        }

        self.primary_device = Some(node);

        Ok(())
    }

    /// Returns the primary DRM device, if one is configured.
    pub fn primary(&self) -> Option<&DrmDeviceInfo> {
        self.primary_device.and_then(|node| self.devices.get(&node))
    }

    /// Returns the identity of the primary DRM device.
    pub fn primary_node(&self) -> Option<DrmNode> {
        self.primary_device
    }

    /// Returns all registered DRM devices.
    pub fn devices(&self) -> impl Iterator<Item = &DrmDeviceInfo> {
        self.devices.values()
    }

    /// Returns the number of registered DRM devices.
    pub fn len(&self) -> usize {
        self.devices.len()
    }

    /// Returns whether the registry contains no devices.
    pub fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_starts_empty() {
        let registry = DrmDeviceRegistry::new();

        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(registry.primary().is_none());
        assert!(registry.primary_node().is_none());
    }
}
