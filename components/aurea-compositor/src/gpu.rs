// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! GPU inventory and device selection management.

#[path = "gpu/discovery.rs"]
mod discovery;

pub use discovery::discover_gpu_selection_metadata;

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

/// Hardware and administrator selection hints used by the primary-GPU policy.
///
/// These values are discovery metadata, not proof that a Vulkan renderer can
/// use the device. The caller must still validate the Vulkan device separately.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GpuSelectionMetadata {
    /// The device was explicitly marked as the preferred primary by system policy.
    pub preferred_primary: bool,
    /// A connected built-in panel (eDP, LVDS or DSI) is attached to this GPU.
    pub has_builtin_panel: bool,
    /// The DRM device is backed by a platform device rather than a PCI device.
    pub is_platform_device: bool,
    /// The firmware identifies this device as the boot VGA device.
    pub is_boot_vga: bool,
    /// Vulkan reports a usable non-CPU renderer for this GPU.
    pub hardware_rendering: bool,
}

/// Metadata tracked for a discovered GPU.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuDevice {
    path: PathBuf,
    availability: GpuAvailability,
    selection: GpuSelectionMetadata,
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

    /// Returns the selection hints discovered for this GPU.
    pub fn selection_metadata(&self) -> GpuSelectionMetadata {
        self.selection
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
                selection: GpuSelectionMetadata::default(),
            },
        );
        Ok(())
    }

    /// Removes a GPU from the inventory and clears selections referring to it.
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

    /// Updates policy metadata after DRM, sysfs, udev and Vulkan discovery.
    pub fn set_selection_metadata(
        &mut self,
        path: &Path,
        metadata: GpuSelectionMetadata,
    ) -> Result<(), GpuManagerError> {
        let device = self
            .devices
            .get_mut(path)
            .ok_or_else(|| GpuManagerError::UnknownDevice(path.to_path_buf()))?;
        device.selection = metadata;
        Ok(())
    }

    /// Chooses a primary GPU using the same priority classes as Mutter.
    ///
    /// The explicit udev preference wins first. Other hardware-rendering
    /// candidates precede software candidates; within each class the order is
    /// built-in panel, platform device, boot VGA, then any remaining device.
    /// Ties are stable
    /// because the inventory is ordered by DRM path.
    ///
    /// This only chooses the compositor's primary GPU. Each output continues
    /// to be owned and presented by its actual DRM device.
    pub fn select_primary_by_policy(&mut self) -> Option<&GpuDevice> {
        // Mutter's explicit udev preference takes precedence even when the
        // selected device needs the software-rendering pass.
        let selected = self
            .devices
            .values()
            .find(|device| device.is_available() && device.selection.preferred_primary)
            .or_else(|| {
                [true, false].into_iter().find_map(|hardware_only| {
                    self.devices
                        .values()
                        .find(|device| {
                            device.is_available()
                                && (!hardware_only || device.selection.hardware_rendering)
                                && device.selection.has_builtin_panel
                        })
                        .or_else(|| {
                            self.devices.values().find(|device| {
                                device.is_available()
                                    && (!hardware_only || device.selection.hardware_rendering)
                                    && device.selection.is_platform_device
                            })
                        })
                        .or_else(|| {
                            self.devices.values().find(|device| {
                                device.is_available()
                                    && (!hardware_only || device.selection.hardware_rendering)
                                    && device.selection.is_boot_vga
                            })
                        })
                        .or_else(|| {
                            self.devices.values().find(|device| {
                                device.is_available()
                                    && (!hardware_only || device.selection.hardware_rendering)
                            })
                        })
                })
            })?;

        self.primary_device = Some(selected.path.clone());
        self.primary_device()
    }

    /// Sets the primary GPU explicitly.
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
    pub fn selected_render_device(&self) -> Option<&GpuDevice> {
        self.preferred_render_device()
            .filter(|device| device.is_available())
            .or_else(|| self.primary_device().filter(|device| device.is_available()))
            .or_else(|| self.devices.values().find(|device| device.is_available()))
    }

    /// Selects a rendering GPU for one output.
    ///
    /// The caller must negotiate the exact rendering/presentation pair.
    /// Cross-GPU compatibility is permitted only through the explicit
    /// host-memory readback path; direct DMA-BUF scanout remains same-GPU.
    pub fn selected_render_device_for_output(
        &self,
        output_device_path: &Path,
    ) -> Option<&GpuDevice> {
        self.selected_render_device_for_output_if(output_device_path, |_| true)
    }

    /// Selects only a GPU whose runtime renderer is actually initialized.
    ///
    /// Discovery and policy metadata alone never make a GPU render-capable.
    /// The caller supplies the live backend check, allowing hotplug and
    /// initialization failures to be reflected without duplicating ownership.
    pub fn selected_render_device_for_output_if<F>(
        &self,
        output_device_path: &Path,
        mut renderer_ready: F,
    ) -> Option<&GpuDevice>
    where
        F: FnMut(&GpuDevice) -> bool,
    {
        let eligible = |device: &&GpuDevice| device.is_available();

        self.preferred_render_device()
            .filter(|device| eligible(device) && renderer_ready(device))
            .or_else(|| {
                self.primary_device()
                    .filter(|device| eligible(device) && renderer_ready(device))
            })
            .or_else(|| {
                self.get_device(output_device_path)
                    .filter(|device| eligible(device) && renderer_ready(device))
            })
            .or_else(|| {
                self.devices
                    .values()
                    .find(|device| eligible(device) && renderer_ready(device))
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
