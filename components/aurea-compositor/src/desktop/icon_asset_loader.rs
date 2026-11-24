// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Loads approved desktop icon bytes from an explicitly supplied asset root.

use std::{
    error::Error,
    fmt,
    fs,
    path::{Component, Path},
};

use crate::render::shm_buffer::{ShmBufferError, ShmBufferSnapshot};

use super::{DesktopIconAssetDescriptor, DesktopIconTextureKey};

/// Failure while resolving or validating a generated desktop icon asset.
#[derive(Debug)]
pub(crate) enum DesktopIconAssetLoadError {
    InvalidRelativePath,
    Io(std::io::Error),
    InvalidPixels(ShmBufferError),
}

impl fmt::Display for DesktopIconAssetLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRelativePath => {
                formatter.write_str("Desktop icon asset path must stay beneath its asset root")
            }
            Self::Io(error) => write!(formatter, "Unable to read desktop icon asset: {error}"),
            Self::InvalidPixels(error) => {
                write!(formatter, "Desktop icon RGBA8 asset is invalid: {error}")
            }
        }
    }
}

impl Error for DesktopIconAssetLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::InvalidPixels(error) => Some(error),
            Self::InvalidRelativePath => None,
        }
    }
}

/// Reads one generated RGBA8 asset beneath an explicit runtime asset directory.
///
/// The caller owns the asset-root policy and should call this outside the frame
/// loop. This function never consults the process working directory and rejects
/// absolute paths, parent traversal and non-normal path components.
pub(crate) fn load_desktop_icon_snapshot(
    asset_root: &Path,
    descriptor: &DesktopIconAssetDescriptor,
) -> Result<(DesktopIconTextureKey, ShmBufferSnapshot), DesktopIconAssetLoadError> {
    let relative_path = Path::new(&descriptor.relative_path);
    if relative_path.as_os_str().is_empty()
        || relative_path.is_absolute()
        || relative_path.components().any(|component| {
            !matches!(component, Component::Normal(_))
        })
    {
        return Err(DesktopIconAssetLoadError::InvalidRelativePath);
    }

    let bytes = fs::read(asset_root.join(relative_path))
        .map_err(DesktopIconAssetLoadError::Io)?;
    let snapshot = descriptor
        .snapshot(&bytes)
        .map_err(DesktopIconAssetLoadError::InvalidPixels)?;

    Ok(snapshot)
}
