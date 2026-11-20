// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Stable hardware-path evidence for physical DRM connectors.

use std::path::{Path, PathBuf};

/// Hardware-path evidence resolved from the Linux DRM sysfs topology.
///
/// This is evidence for a physical connector, not a globally unique display ID.
/// EDID evidence and duplicate detection must be evaluated separately.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DrmConnectorIdentity {
    gpu_sysfs_path: PathBuf,
    connector_name: String,
}

impl DrmConnectorIdentity {
    /// Resolves a connector's GPU path through its sysfs device link.
    ///
    /// device_path is the Udev path used to open /dev/dri/cardN.
    /// The connector name is the kernel interface name, e.g. DP-1 or
    /// DP-1-2 for an MST branch. Volatile cardN numbering is not retained.
    pub fn resolve(
        device_path: &Path,
        connector_name: &str,
        sysfs_root: &Path,
    ) -> Option<Self> {
        let card_name = device_path.file_name()?.to_str()?;
        if !is_card_name(card_name) || !is_valid_connector_name(connector_name) {
            return None;
        }

        let device_link = sysfs_root
            .join("class/drm")
            .join(format!("{card_name}-{connector_name}"))
            .join("device");
        let gpu_sysfs_path = std::fs::canonicalize(device_link).ok()?;

        Some(Self {
            gpu_sysfs_path,
            connector_name: connector_name.to_owned(),
        })
    }

    /// Returns the canonical GPU sysfs path, normally a PCI device path.
    pub fn gpu_sysfs_path(&self) -> &Path {
        &self.gpu_sysfs_path
    }

    /// Returns the connector port name, preserving MST topology suffixes.
    pub fn connector_name(&self) -> &str {
        &self.connector_name
    }
}

fn is_card_name(value: &str) -> bool {
    value.strip_prefix("card").is_some_and(|suffix| {
        !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
    })
}

fn is_valid_connector_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        && value.as_bytes()[0].is_ascii_alphabetic()
}

#[cfg(test)]
mod tests {
    use super::{is_card_name, is_valid_connector_name};

    #[test]
    fn accepts_kernel_card_device_names() {
        assert!(is_card_name("card0"));
        assert!(is_card_name("card12"));
    }

    #[test]
    fn rejects_non_card_device_names() {
        assert!(!is_card_name("renderD128"));
        assert!(!is_card_name("card"));
        assert!(!is_card_name("card0-DP-1"));
    }

    #[test]
    fn accepts_connector_names_and_mst_suffixes() {
        assert!(is_valid_connector_name("DP-1"));
        assert!(is_valid_connector_name("DP-1-2"));
        assert!(is_valid_connector_name("HDMI-A-1"));
    }

    #[test]
    fn rejects_connector_path_injection() {
        assert!(!is_valid_connector_name("../device"));
        assert!(!is_valid_connector_name("DP/1"));
        assert!(!is_valid_connector_name(""));
    }
}
