
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Native DRM/KMS backend modules.

pub mod device;
pub mod device_manager;
pub mod discovery;
pub mod gbm;
pub mod output;
pub mod outputs;
pub mod presentation;
pub mod presentation_backend;
pub mod plane_formats;
pub mod registry;
pub mod session;
pub mod surfaces;

pub use plane_formats::{
    DrmPlaneFormatError,
    DrmPrimaryPlaneFormatCapabilities,
    discover_primary_plane_formats,
};
pub use session::{DrmSession, DrmSessionError};
