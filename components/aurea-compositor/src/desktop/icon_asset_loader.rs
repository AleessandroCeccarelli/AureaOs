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
    AssetOutsideRoot,
    UnexpectedFileLength { expected: u64, actual: u64 },
    Io(std::io::Error),
    InvalidPixels(ShmBufferError),
}

impl fmt::Display for DesktopIconAssetLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRelativePath => {
                formatter.write_str("Desktop icon asset path must be relative and normalized")
            }
            Self::AssetOutsideRoot => {
                formatter.write_str("Desktop icon asset resolves outside its asset root")
            }
            Self::UnexpectedFileLength { expected, actual } => write!(
                formatter,
                "Desktop icon RGBA8 asset has {actual} bytes; expected exactly {expected}"
            ),
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
            Self::InvalidRelativePath
            | Self::AssetOutsideRoot
            | Self::UnexpectedFileLength { .. } => None,
        }
    }
}

/// Reads one generated RGBA8 asset beneath an explicit runtime asset directory.
///
/// The caller owns the asset-root policy and should call this outside the frame
/// loop. This function never consults the process working directory. It rejects
/// non-normal relative paths, symlink escapes and files whose length does not
/// exactly match the descriptor before allocating their contents.
pub(crate) fn load_desktop_icon_snapshot(
    asset_root: &Path,
    descriptor: &DesktopIconAssetDescriptor,
) -> Result<(DesktopIconTextureKey, ShmBufferSnapshot), DesktopIconAssetLoadError> {
    let relative_path = Path::new(&descriptor.relative_path);
    if relative_path.as_os_str().is_empty()
        || relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        || descriptor.key.pixel_size == 0
        || descriptor.key.pixel_size > 256
    {
        return Err(DesktopIconAssetLoadError::InvalidRelativePath);
    }

    let expected_len = u64::from(descriptor.key.pixel_size)
        .checked_mul(u64::from(descriptor.key.pixel_size))
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(DesktopIconAssetLoadError::InvalidRelativePath)?;

    let canonical_root =
        fs::canonicalize(asset_root).map_err(DesktopIconAssetLoadError::Io)?;
    let candidate = canonical_root.join(relative_path);
    let canonical_asset =
        fs::canonicalize(candidate).map_err(DesktopIconAssetLoadError::Io)?;
    if !canonical_asset.starts_with(&canonical_root) {
        return Err(DesktopIconAssetLoadError::AssetOutsideRoot);
    }

    let metadata = fs::metadata(&canonical_asset).map_err(DesktopIconAssetLoadError::Io)?;
    let actual_len = metadata.len();
    if actual_len != expected_len {
        return Err(DesktopIconAssetLoadError::UnexpectedFileLength {
            expected: expected_len,
            actual: actual_len,
        });
    }

    let bytes = fs::read(canonical_asset).map_err(DesktopIconAssetLoadError::Io)?;
    let snapshot = descriptor
        .snapshot(&bytes)
        .map_err(DesktopIconAssetLoadError::InvalidPixels)?;

    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{DesktopIconAssetLoadError, load_desktop_icon_snapshot};
    use crate::desktop::{DesktopIconAssetDescriptor, DesktopIconTextureKey};

    fn temporary_root() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        std::env::temp_dir().join(format!("aurea-icon-loader-{nonce}"))
    }

    fn descriptor(path: &str) -> DesktopIconAssetDescriptor {
        DesktopIconAssetDescriptor {
            key: DesktopIconTextureKey {
                asset_id: "test:icon",
                revision: 1,
                pixel_size: 1,
            },
            relative_path: path.to_owned(),
        }
    }

    #[test]
    fn loads_only_an_exact_rgba8_payload() {
        let root = temporary_root();
        fs::create_dir_all(&root).expect("create root");
        fs::write(root.join("icon.rgba"), [1_u8, 2, 3, 4]).expect("write asset");

        let loaded = load_desktop_icon_snapshot(&root, &descriptor("icon.rgba"));
        assert!(loaded.is_ok());

        fs::remove_dir_all(root).expect("remove root");
    }

    #[test]
    fn rejects_parent_traversal_before_filesystem_access() {
        let root = temporary_root();
        fs::create_dir_all(&root).expect("create root");

        let result = load_desktop_icon_snapshot(&root, &descriptor("../icon.rgba"));
        assert!(matches!(
            result,
            Err(DesktopIconAssetLoadError::InvalidRelativePath)
        ));

        fs::remove_dir_all(root).expect("remove root");
    }

    #[test]
    fn rejects_files_larger_or_smaller_than_the_descriptor() {
        let root = temporary_root();
        fs::create_dir_all(&root).expect("create root");
        fs::write(root.join("icon.rgba"), [1_u8, 2, 3, 4, 5]).expect("write asset");

        let result = load_desktop_icon_snapshot(&root, &descriptor("icon.rgba"));
        assert!(matches!(
            result,
            Err(DesktopIconAssetLoadError::UnexpectedFileLength {
                expected: 4,
                actual: 5
            })
        ));

        fs::remove_dir_all(root).expect("remove root");
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinks_that_escape_the_asset_root() {
        use std::os::unix::fs::symlink;

        let root = temporary_root();
        let outside = temporary_root();
        fs::create_dir_all(&root).expect("create root");
        fs::create_dir_all(&outside).expect("create outside");
        fs::write(outside.join("icon.rgba"), [1_u8, 2, 3, 4]).expect("write outside asset");
        symlink(outside.join("icon.rgba"), root.join("icon.rgba")).expect("create symlink");

        let result = load_desktop_icon_snapshot(&root, &descriptor("icon.rgba"));
        assert!(matches!(
            result,
            Err(DesktopIconAssetLoadError::AssetOutsideRoot)
        ));

        fs::remove_dir_all(root).expect("remove root");
        fs::remove_dir_all(outside).expect("remove outside");
    }
}
