// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! State shared by Calloop event callbacks.

use std::{
    io,
    sync::{Arc, Mutex},
};

#[cfg(feature = "drm")]
use std::{collections::HashMap, path::PathBuf};

#[cfg(feature = "drm")]
use smithay::backend::drm::DrmNode;

#[cfg(feature = "drm")]
use smithay::reexports::calloop::{LoopHandle, RegistrationToken};

use smithay::reexports::wayland_server::{Client, Display, backend::ClientId};

use crate::AureaCompositor;

#[cfg(feature = "drm")]
use crate::{
    drm::{
    DrmSession,
    device_manager::DrmDeviceManager,
    gbm::GbmDeviceManager,
    registry::DrmDeviceRegistry,
},
    gpu::GpuManager,
    vulkan::VulkanBackend,
};

/// Runtime state accessed by the server event loop.
pub(super) struct WaylandState {
    pub(super) display: Display<AureaCompositor>,
    pub(super) compositor: AureaCompositor,
    pub(super) clients: Vec<Client>,
    pub(super) disconnected_clients: Arc<Mutex<Vec<ClientId>>>,
    pub(super) error: Option<io::Error>,

    #[cfg(feature = "drm")]
    pub(super) drm_session: Option<DrmSession>,

    #[cfg(feature = "drm")]
    pub(super) drm_devices: DrmDeviceManager,

    /// GBM buffer allocation devices keyed by their DRM identity.
#[cfg(feature = "drm")]
pub(super) gbm_devices: GbmDeviceManager,

    /// Calloop source handles keyed by the corresponding DRM device.
    #[cfg(feature = "drm")]
    pub(super) drm_notifiers: HashMap<DrmNode, RegistrationToken>,

    #[cfg(feature = "drm")]
    pub(super) loop_handle: Option<LoopHandle<'static, WaylandState>>,

    #[cfg(feature = "drm")]
    pub(super) session_active: bool,

    /// GPU inventory and rendering preference manager.
    #[cfg(feature = "drm")]
    pub(super) gpu_manager: GpuManager,

    /// Canonical registry of discovered DRM devices.
    #[cfg(feature = "drm")]
    pub(super) drm_device_registry: DrmDeviceRegistry,

    /// Maps Udev DRM device identities to their device paths.
    #[cfg(feature = "drm")]
    pub(super) udev_device_ids: Vec<(DrmNode, PathBuf)>,

    /// Changed DRM devices waiting for output discovery.
    #[cfg(feature = "drm")]
    pub(super) udev_changed_devices: Vec<DrmNode>,

    /// Vulkan rendering backends keyed by their associated DRM device.
    ///
    /// Each backend is responsible for one GPU. Output-to-device association
    /// is derived from the DRM identity encoded in each output ID.
    #[cfg(feature = "drm")]
    pub(super) vulkan_backends: HashMap<DrmNode, VulkanBackend>,
}
