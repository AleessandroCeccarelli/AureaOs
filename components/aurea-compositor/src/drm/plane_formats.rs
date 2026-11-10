// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! DRM/KMS primary-plane format discovery.
//!
//! This module deliberately uses Smithay's DRM plane inventory instead of
//! parsing the kernel's IN_FORMATS property blob a second time. Smithay
//! already translates the kernel data into format/modifier pairs.

use std::{error::Error, fmt};

use smithay::{
    backend::{
        allocator::Format,
        drm::DrmDevice as SmithayDrmDevice,
    },
    reexports::drm::control::{PlaneType, crtc},
};

use crate::drm::device::DrmDevice;

/// Format and modifier capabilities reported for one primary DRM plane.
///
/// The identifiers are local to the DRM device. They must always be used
/// together with the owning DRM device identity; they are not globally unique.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrmPrimaryPlaneFormatCapabilities {
    /// CRTC identifier to which this plane is compatible.
    pub crtc_id: u32,

    /// DRM plane identifier.
    pub plane_id: u32,

    /// Format and modifier pairs reported by Smithay for this plane.
    pub formats: Vec<Format>,
}

impl DrmPrimaryPlaneFormatCapabilities {
    /// Returns whether this plane reports the specified format/modifier pair.
    pub fn supports(&self, format: Format) -> bool {
        self.formats.contains(&format)
    }
}

/// Errors produced while discovering primary-plane formats.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrmPlaneFormatError {
    /// The DRM device is paused or inactive.
    DeviceInactive,

    /// Smithay could not query the planes compatible with the CRTC.
    PlaneQueryFailed {
        crtc_id: u32,
        message: String,
    },

    /// No primary plane was reported for the requested CRTC.
    NoPrimaryPlane {
        crtc_id: u32,
    },
}

impl fmt::Display for DrmPlaneFormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeviceInactive => {
                formatter.write_str(
                    "Cannot discover DRM plane formats while the device is inactive",
                )
            }
            Self::PlaneQueryFailed { crtc_id, message } => {
                write!(
                    formatter,
                    "Failed to query planes for CRTC {crtc_id}: {message}"
                )
            }
            Self::NoPrimaryPlane { crtc_id } => {
                write!(formatter, "No primary DRM plane is available for CRTC {crtc_id}")
            }
        }
    }
}

impl Error for DrmPlaneFormatError {}

/// Discovers the format/modifier pairs of primary planes compatible with a CRTC.
///
/// Smithay performs the kernel plane and IN_FORMATS property queries and
/// exposes the resulting pairs through PlaneInfo::formats.
///
/// This is a capability inventory, not a guarantee that every framebuffer
/// configuration will succeed. Final compatibility can still depend on
/// dimensions, scaling, other plane state and an atomic test-only commit.
///
/// The returned CRTC and plane identifiers are local to the supplied DRM
/// device. Do not combine them with identifiers or capabilities from another GPU.
pub fn discover_primary_plane_formats(
    device: &DrmDevice,
    crtc: crtc::Handle,
) -> Result<Vec<DrmPrimaryPlaneFormatCapabilities>, DrmPlaneFormatError> {
    if !device.is_active() {
        return Err(DrmPlaneFormatError::DeviceInactive);
    }

    let crtc_id = u32::from(crtc);
    let planes = device
        .device()
        .planes(&crtc)
        .map_err(|error| DrmPlaneFormatError::PlaneQueryFailed {
            crtc_id,
            message: error.to_string(),
        })?;

    if planes.primary.is_empty() {
        return Err(DrmPlaneFormatError::NoPrimaryPlane { crtc_id });
    }

    Ok(planes
        .primary
        .into_iter()
        .filter(|plane| plane.type_ == PlaneType::Primary)
        .map(|plane| DrmPrimaryPlaneFormatCapabilities {
            crtc_id,
            plane_id: u32::from(plane.handle),
            formats: plane.formats.iter().copied().collect(),
        })
        .collect())
}
