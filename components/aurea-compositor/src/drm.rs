// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Native DRM/KMS backend modules.

pub mod connector_identity;
pub(crate) mod cpu_presentation_backend;
pub mod device;
pub mod device_manager;
pub mod discovery;
pub mod edid;
pub mod edid_hdr;
pub mod frame_coordinator;
pub mod gbm;
pub(crate) mod gbm_buffer_allocator;
pub mod gbm_buffer_writer;
pub(crate) mod gbm_framebuffer;
pub(crate) mod gbm_framebuffer_pool;
pub mod gbm_upload_layout;
pub mod hdr_link;
pub mod identity;
pub mod identity_registry;
pub mod output;
pub mod outputs;
pub mod plane_formats;
pub mod presentation;
pub mod presentation_backend;
pub mod registry;
pub mod session;
pub mod surfaces;

pub use plane_formats::{
    DrmPlaneFormatError, DrmPrimaryPlaneFormatCapabilities, discover_primary_plane_formats,
};
pub use session::{DrmSession, DrmSessionError};

pub(crate) mod scanout_renderer;
