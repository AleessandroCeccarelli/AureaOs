// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Reads GPU selection hints from Linux sysfs and the udev database.

use std::{fs, io, path::Path};

use super::GpuSelectionMetadata;

/// Reads Mutter-compatible primary-GPU hints for a DRM card.
///
/// Missing sysfs or udev attributes are treated as absent hints: discovery
/// must remain non-fatal on kernels and systems exposing partial metadata.
pub fn discover_gpu_selection_metadata(device_path: &Path) -> io::Result<GpuSelectionMetadata> {
    discover_gpu_selection_metadata_from(
        device_path,
        Path::new("/sys"),
        Path::new("/run/udev/data"),
    )
}

fn discover_gpu_selection_metadata_from(
    device_path: &Path,
    sysfs_root: &Path,
    udev_data_root: &Path,
) -> io::Result<GpuSelectionMetadata> {
    let card_name = device_path
        .file_name()
        .filter(|name| name.to_string_lossy().starts_with("card"))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "not a DRM card path"))?;

    let card_sysfs = sysfs_root.join("class/drm").join(card_name);
    let device_sysfs = card_sysfs.join("device");

    let preferred_primary = read_device_number(&card_sysfs.join("dev"))
        .map(|dev| read_udev_preferred_primary(udev_data_root, &dev))
        .unwrap_or(false);

    let is_platform_device = fs::canonicalize(device_sysfs.join("subsystem"))
        .map(|path| path.to_string_lossy().contains("/bus/platform"))
        .unwrap_or(false);

    let is_boot_vga =
        read_trimmed(&device_sysfs.join("boot_vga")).is_some_and(|value| value == "1");

    let has_builtin_panel = fs::read_dir(sysfs_root.join("class/drm"))
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            name.starts_with(&format!("{}-", card_name.to_string_lossy()))
                && ["eDP-", "LVDS-", "DSI-"]
                    .iter()
                    .any(|kind| name.contains(kind))
        })
        .any(|entry| read_trimmed(&entry.path().join("status")).as_deref() == Some("connected"));

    Ok(GpuSelectionMetadata {
        preferred_primary,
        has_builtin_panel,
        is_platform_device,
        is_boot_vga,
        hardware_rendering: false,
    })
}

fn read_device_number(path: &Path) -> Option<String> {
    read_trimmed(path)
}

fn read_udev_preferred_primary(udev_data_root: &Path, device_number: &str) -> bool {
    let database_path = udev_data_root.join(format!("c{device_number}"));
    fs::read_to_string(database_path)
        .map(|contents| {
            contents.lines().any(|line| {
                line == "G:mutter-device-preferred-primary"
                    || line == "E:MUTTER_DEVICE_PREFERRED_PRIMARY=1"
            })
        })
        .unwrap_or(false)
}

fn read_trimmed(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_owned())
}

#[cfg(test)]
#[path = "../../tests/unit/gpu_discovery.rs"]
mod tests;
