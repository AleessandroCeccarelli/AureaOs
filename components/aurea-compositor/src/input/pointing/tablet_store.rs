// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Durable pen-tablet calibration profiles keyed by physical device identity.

use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use super::{TabletAspectMode, TabletCalibration, TabletRotation};

const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum StoredRotation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum StoredAspectMode {
    Stretch,
    Preserve,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct StoredTabletProfile {
    active_area: [f64; 4],
    rotation: StoredRotation,
    aspect_mode: StoredAspectMode,
    device_aspect_ratio: f64,
}

impl From<TabletCalibration> for StoredTabletProfile {
    fn from(profile: TabletCalibration) -> Self {
        Self {
            active_area: profile.active_area(),
            rotation: match profile.rotation() {
                TabletRotation::Deg0 => StoredRotation::Deg0,
                TabletRotation::Deg90 => StoredRotation::Deg90,
                TabletRotation::Deg180 => StoredRotation::Deg180,
                TabletRotation::Deg270 => StoredRotation::Deg270,
            },
            aspect_mode: match profile.aspect_mode() {
                TabletAspectMode::Stretch => StoredAspectMode::Stretch,
                TabletAspectMode::Preserve => StoredAspectMode::Preserve,
            },
            device_aspect_ratio: profile.device_aspect_ratio(),
        }
    }
}

impl TryFrom<StoredTabletProfile> for TabletCalibration {
    type Error = io::Error;

    fn try_from(profile: StoredTabletProfile) -> Result<Self, Self::Error> {
        let rotation = match profile.rotation {
            StoredRotation::Deg0 => TabletRotation::Deg0,
            StoredRotation::Deg90 => TabletRotation::Deg90,
            StoredRotation::Deg180 => TabletRotation::Deg180,
            StoredRotation::Deg270 => TabletRotation::Deg270,
        };
        let aspect_mode = match profile.aspect_mode {
            StoredAspectMode::Stretch => TabletAspectMode::Stretch,
            StoredAspectMode::Preserve => TabletAspectMode::Preserve,
        };
        TabletCalibration::new(
            profile.active_area,
            rotation,
            aspect_mode,
            profile.device_aspect_ratio,
        )
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid stored tablet calibration",
            )
        })
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct StoredProfiles {
    format_version: u32,
    devices: HashMap<String, StoredTabletProfile>,
}

/// Loads and safely persists per-tablet calibration independently of runtime IDs.
pub struct TabletProfileStore {
    path: Option<PathBuf>,
    profiles: HashMap<String, TabletCalibration>,
    writable: bool,
}

impl Default for TabletProfileStore {
    fn default() -> Self {
        match default_store_path() {
            Some(path) => Self::open(path),
            None => Self {
                path: None,
                profiles: HashMap::new(),
                writable: true,
            },
        }
    }
}

impl TabletProfileStore {
    #[cfg(test)]
    fn with_path(path: PathBuf) -> Self {
        Self::open(path)
    }

    fn open(path: PathBuf) -> Self {
        match load_profiles(&path) {
            Ok(profiles) => Self {
                path: Some(path),
                profiles,
                writable: true,
            },
            Err(error) => {
                tracing::warn!(
                    path = %path.display(),
                    %error,
                    "Tablet preferences opened read-only; original file will be preserved"
                );
                Self {
                    path: Some(path),
                    profiles: HashMap::new(),
                    writable: false,
                }
            }
        }
    }

    pub fn get(&self, identity: &str) -> Option<TabletCalibration> {
        self.profiles.get(identity).copied()
    }

    pub fn set(&mut self, identity: String, profile: TabletCalibration) -> io::Result<()> {
        let previous = self.profiles.insert(identity.clone(), profile);
        if let Err(error) = self.persist() {
            if let Some(previous) = previous {
                self.profiles.insert(identity, previous);
            } else {
                self.profiles.remove(&identity);
            }
            return Err(error);
        }
        Ok(())
    }

    pub fn remove(&mut self, identity: &str) -> io::Result<bool> {
        let Some(previous) = self.profiles.remove(identity) else {
            return Ok(false);
        };
        if let Err(error) = self.persist() {
            self.profiles.insert(identity.to_string(), previous);
            return Err(error);
        }
        Ok(true)
    }

    fn persist(&self) -> io::Result<()> {
        if !self.writable {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "tablet preferences were not loaded safely; refusing to overwrite the original file",
            ));
        }
        let Some(path) = self.path.as_deref() else {
            return Ok(());
        };
        let parent = path.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "tablet settings path has no parent",
            )
        })?;
        fs::create_dir_all(parent)?;

        let document = StoredProfiles {
            format_version: FORMAT_VERSION,
            devices: self
                .profiles
                .iter()
                .map(|(identity, profile)| (identity.clone(), (*profile).into()))
                .collect(),
        };
        let encoded = toml::to_string_pretty(&document)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let temporary = path.with_extension("toml.tmp");
        let mut options = OpenOptions::new();
        options.create(true).truncate(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temporary)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(fs::Permissions::from_mode(0o600))?;
        }
        file.write_all(encoded.as_bytes())?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)?;
        if let Ok(directory) = OpenOptions::new().read(true).open(parent) {
            let _ = directory.sync_all();
        }
        Ok(())
    }
}

fn default_store_path() -> Option<PathBuf> {
    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        if !config_home.is_empty() {
            return Some(PathBuf::from(config_home).join("aureaos/input/tablets.toml"));
        }
    }
    std::env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(|home| PathBuf::from(home).join(".config/aureaos/input/tablets.toml"))
}

fn load_profiles(path: &Path) -> io::Result<HashMap<String, TabletCalibration>> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(HashMap::new()),
        Err(error) => return Err(error),
    };
    let stored: StoredProfiles = toml::from_str(&content)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    if stored.format_version != FORMAT_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "unsupported tablet settings format {}; expected {}",
                stored.format_version, FORMAT_VERSION
            ),
        ));
    }
    stored
        .devices
        .into_iter()
        .map(|(identity, profile)| Ok((identity, profile.try_into()?)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    fn test_path(suffix: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "aurea-tablet-profiles-{}-{suffix}.toml",
            std::process::id()
        ))
    }

    fn profile(rotation: TabletRotation) -> TabletCalibration {
        TabletCalibration::new(
            [0.1, 0.2, 0.7, 0.6],
            rotation,
            TabletAspectMode::Preserve,
            1.6,
        )
        .unwrap()
    }

    #[test]
    fn profiles_survive_recreation_and_keep_tablets_separate() {
        let path = test_path("roundtrip");
        let _ = fs::remove_file(&path);
        let mut first = TabletProfileStore::with_path(path.clone());
        let a = profile(TabletRotation::Deg90);
        let b = profile(TabletRotation::Deg270);
        first.set("tablet-a".into(), a).unwrap();
        first.set("tablet-b".into(), b).unwrap();

        let second = TabletProfileStore::with_path(path.clone());
        assert_eq!(second.get("tablet-a"), Some(a));
        assert_eq!(second.get("tablet-b"), Some(b));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn malformed_or_future_files_are_never_overwritten() {
        for (suffix, original) in [
            ("malformed", "not valid TOML = ["),
            ("future", "format_version = 99\n[devices]\n"),
        ] {
            let path = test_path(suffix);
            fs::write(&path, original).unwrap();
            let mut store = TabletProfileStore::with_path(path.clone());
            assert!(
                store
                    .set("tablet-a".into(), profile(TabletRotation::Deg0))
                    .is_err()
            );
            assert_eq!(fs::read_to_string(&path).unwrap(), original);
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn invalid_calibration_is_rejected_without_overwriting_file() {
        let path = test_path("invalid-profile");
        let original = "[devices.tablet-a]\nactive_area = [0.0, 0.0, 2.0, 1.0]\nrotation = \"deg0\"\naspect_mode = \"stretch\"\ndevice_aspect_ratio = 1.0\n";
        fs::write(&path, original).unwrap();
        let mut store = TabletProfileStore::with_path(path.clone());
        assert!(
            store
                .set("tablet-a".into(), profile(TabletRotation::Deg0))
                .is_err()
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), original);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn remove_persists_reset() {
        let path = test_path("remove");
        let _ = fs::remove_file(&path);
        let mut store = TabletProfileStore::with_path(path.clone());
        store
            .set("tablet-a".into(), profile(TabletRotation::Deg180))
            .unwrap();
        assert!(store.remove("tablet-a").unwrap());
        assert!(
            TabletProfileStore::with_path(path.clone())
                .get("tablet-a")
                .is_none()
        );
        let _ = fs::remove_file(path);
    }
}
