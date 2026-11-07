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
use std::path::PathBuf;

use smithay::reexports::wayland_server::{Client, Display, backend::ClientId};

#[cfg(feature = "drm")]
use smithay::backend::udev::UdevEvent;

use crate::AureaCompositor;

#[cfg(feature = "drm")]
use crate::drm::{DrmSession, device::DrmDevice};

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
    pub(super) _drm_device: Option<DrmDevice>,

    #[cfg(feature = "drm")]
    pub(super) session_active: bool,

    /// DRM device paths discovered during initial Udev enumeration.
    #[cfg(feature = "drm")]
    pub(super) udev_device_paths: Vec<PathBuf>,

    /// Udev events waiting for DRM device lifecycle processing.
    #[cfg(feature = "drm")]
    pub(super) udev_events: Vec<UdevEvent>,
}
