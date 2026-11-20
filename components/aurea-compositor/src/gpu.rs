// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! GPU inventory and device selection management.

use std::{
    collections::BTreeMap,
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

/// Runtime availability of a discovered GPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuAvailability {
    /// The device is available for use.
    Available,

    /// The device is temporarily unavailable.
    Unavailable,
}

/// Metadata tracked for a discovered GPU.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuDevice {
    path: PathBuf,
    availability: GpuAvailability,
}

impl GpuDevice {
    /// Returns the DRM device path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the current availability.
    pub fn availability(&self) -> GpuAvailability {
        self.availability
    }

    /// Returns whether the device is currently available.
    pub fn is_available(&self) -> bool {
        self.availability == GpuAvailability::Available
    }
}

/// Errors returned by the GPU manager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuManagerError {
    /// The supplied device path is not absolute.
    InvalidDevicePath(PathBuf),

    /// The device is already registered.
    DeviceAlreadyRegistered(PathBuf),

    /// The requested device is not registered.
    UnknownDevice(PathBuf),
}

impl fmt::Display for GpuManagerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDevicePath(path) => {
                write!(formatter, "Invalid GPU device path: {}", path.display())
            }
            Self::DeviceAlreadyRegistered(path) => {
                write!(
                    formatter,
                    "GPU device is already registered: {}",
                    path.display()
                )
            }
            Self::UnknownDevice(path) => {
                write!(formatter, "Unknown GPU device: {}", path.display())
            }
        }
    }
}

impl Error for GpuManagerError {}

/// Tracks discovered GPUs and the compositor's device preferences.
///
/// This manager owns device inventory and selection metadata only.
/// Opening DRM devices and creating renderers are handled by other components.
#[derive(Debug, Default)]
pub struct GpuManager {
    devices: BTreeMap<PathBuf, GpuDevice>,
    primary_device: Option<PathBuf>,
    preferred_render_device: Option<PathBuf>,
}

impl GpuManager {
    /// Creates an empty GPU manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a discovered DRM device.
    ///
    /// New devices are initially considered available.
    /// Duplicate paths are rejected.
    pub fn register_device(&mut self, path: impl Into<PathBuf>) -> Result<(), GpuManagerError> {
        let path = path.into();

        if !path.is_absolute() {
            return Err(GpuManagerError::InvalidDevicePath(path));
        }

        if self.devices.contains_key(&path) {
            return Err(GpuManagerError::DeviceAlreadyRegistered(path));
        }

        self.devices.insert(
            path.clone(),
            GpuDevice {
                path,
                availability: GpuAvailability::Available,
            },
        );

        Ok(())
    }

    /// Removes a GPU from the inventory.
    ///
    /// Any primary or preferred selection referring to this device is cleared.
    /// Returns the removed device when it was registered.
    pub fn remove_device(&mut self, path: &Path) -> Option<GpuDevice> {
        let removed = self.devices.remove(path)?;

        if self.primary_device.as_deref() == Some(path) {
            self.primary_device = None;
        }

        if self.preferred_render_device.as_deref() == Some(path) {
            self.preferred_render_device = None;
        }

        Some(removed)
    }

    /// Updates the availability of a registered GPU.
    pub fn set_device_availability(
        &mut self,
        path: &Path,
        availability: GpuAvailability,
    ) -> Result<(), GpuManagerError> {
        let device = self
            .devices
            .get_mut(path)
            .ok_or_else(|| GpuManagerError::UnknownDevice(path.to_path_buf()))?;

        device.availability = availability;

        Ok(())
    }

    /// Sets the primary GPU.
    pub fn set_primary_device(&mut self, path: impl AsRef<Path>) -> Result<(), GpuManagerError> {
        let path = path.as_ref();

        if !self.devices.contains_key(path) {
            return Err(GpuManagerError::UnknownDevice(path.to_path_buf()));
        }

        self.primary_device = Some(path.to_path_buf());

        Ok(())
    }

    /// Returns the configured primary GPU, if registered.
    pub fn primary_device(&self) -> Option<&GpuDevice> {
        self.primary_device
            .as_ref()
            .and_then(|path| self.devices.get(path))
    }

    /// Sets an explicit preferred rendering GPU.
    pub fn set_preferred_render_device(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<(), GpuManagerError> {
        let path = path.as_ref();

        if !self.devices.contains_key(path) {
            return Err(GpuManagerError::UnknownDevice(path.to_path_buf()));
        }

        self.preferred_render_device = Some(path.to_path_buf());

        Ok(())
    }

    /// Clears the explicit rendering GPU preference.
    pub fn clear_preferred_render_device(&mut self) {
        self.preferred_render_device = None;
    }

    /// Returns the explicitly preferred rendering GPU, if configured.
    pub fn preferred_render_device(&self) -> Option<&GpuDevice> {
        self.preferred_render_device
            .as_ref()
            .and_then(|path| self.devices.get(path))
    }

    /// Selects an available rendering GPU.
    ///
    /// Selection order:
    /// 1. Explicit preferred GPU, when available.
    /// 2. Primary GPU, when available.
    /// 3. First available registered GPU.
    ///
    /// This is a deterministic fallback policy, not a performance benchmark.
    pub fn selected_render_device(&self) -> Option<&GpuDevice> {
        self.preferred_render_device()
            .filter(|device| device.is_available())
            .or_else(|| self.primary_device().filter(|device| device.is_available()))
            .or_else(|| self.devices.values().find(|device| device.is_available()))
    }

    /// Selects a rendering GPU for one output without choosing an unrelated device.
    ///
    /// Selection order:
    /// 1. Explicit preferred GPU, when available.
    /// 2. Primary GPU, when available.
    /// 3. The GPU driving this output, when available.
    ///
    /// Unlike selected_render_device, this method never falls back to an
    /// arbitrary inventory entry that may not be able to present this output.
    pub fn selected_render_device_for_output(
        &self,
        output_device_path: &Path,
    ) -> Option<&GpuDevice> {
        self.preferred_render_device()
            .filter(|device| device.is_available())
            .or_else(|| self.primary_device().filter(|device| device.is_available()))
            .or_else(|| {
                self.get_device(output_device_path)
                    .filter(|device| device.is_available())
            })
    }

    /// Returns a registered GPU by its DRM path.
    pub fn get_device(&self, path: &Path) -> Option<&GpuDevice> {
        self.devices.get(path)
    }

    /// Returns all registered GPUs in deterministic path order.
    pub fn devices(&self) -> impl Iterator<Item = &GpuDevice> {
        self.devices.values()
    }

    /// Returns the number of registered GPUs.
    pub fn len(&self) -> usize {
        self.devices.len()
    }

    /// Returns whether the inventory is empty.
    pub fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }
}

#[cfg(test)]
#[path = "../tests/unit/gpu.rs"]
mod tests;
