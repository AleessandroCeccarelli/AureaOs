// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Native session and DRM device initialization.

#[cfg(feature = "drm")]
use crate::drm::{DrmSession, device::DrmDevice};

#[cfg(feature = "drm")]
use std::{io, path::Path};

/// Creates a native libseat session.
#[cfg(feature = "drm")]
pub(super) fn create_session() -> io::Result<DrmSession> {
    DrmSession::new().map_err(io::Error::other)
}

/// Opens the primary GPU associated with the requested seat.
#[cfg(feature = "drm")]
pub(super) fn open_primary_device(session: &mut DrmSession, seat: &str) -> io::Result<DrmDevice> {
    DrmDevice::open_primary(session, seat).map_err(io::Error::other)
}

/// Opens a DRM device at the supplied path.
#[cfg(feature = "drm")]
pub(super) fn open_device(
    session: &mut DrmSession,
    device_path: impl AsRef<Path>,
) -> io::Result<DrmDevice> {
    DrmDevice::open(session, device_path).map_err(io::Error::other)
}
