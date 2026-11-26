// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Durable mouse and pointer preferences keyed by physical device identity.

use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use aurea_desktop_api::input::{PointerAccelerationProfile, PointerSettings};
use serde::{Deserialize, Serialize};

const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum StoredAccelerationProfile {
    Adaptive,
    Flat,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct StoredPointerSettings {
    profile: StoredAccelerationProfile,
    speed: f64,
    shake_to_locate: bool,
}

impl From<PointerSettings> for StoredPointerSettings {
    fn from(settings: PointerSettings) -> Self {
        Self {
            profile: match settings.profile() {
                PointerAccelerationProfile::Adaptive => StoredAccelerationProfile::Adaptive,
                PointerAccelerationProfile::Flat => StoredAccelerationProfile::Flat,
            },
            speed: settings.speed(),
            shake_to_locate: settings.shake_to_locate(),
        }
    }
}

impl TryFrom<StoredPointerSettings> for PointerSettings {
    type Error = io::Error;

    fn try_from(settings: StoredPointerSettings) -> Result<Self, Self::Error> {
        let profile = match settings.profile {
            StoredAccelerationProfile::Adaptive => PointerAccelerationProfile::Adaptive,
            StoredAccelerationProfile::Flat => PointerAccelerationProfile::Flat,
        };
        PointerSettings::new(profile, settings.speed, settings.shake_to_locate).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "invalid stored pointer speed")
        })
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct StoredProfiles {
    format_version: u32,
    devices: HashMap<String, StoredPointerSettings>,
}

/// Stores user preferences independently from transient libinput device IDs.
pub struct PointerProfileStore {
    path: Option<PathBuf>,
    profiles: HashMap<String, PointerSettings>,
    writable: bool,
}

impl Default for PointerProfileStore {
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

impl PointerProfileStore {
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
                    "Pointer preferences opened read-only; original file will be preserved"
                );
                Self {
                    path: Some(path),
                    profiles: HashMap::new(),
                    writable: false,
                }
            }
        }
    }

    pub fn get(&self, identity: &str) -> Option<PointerSettings> {
        self.profiles.get(identity).copied()
    }

    /// Updates the in-memory profile and persists it when the store is writable.
    pub fn set(&mut self, identity: String, settings: PointerSettings) -> io::Result<()> {
        if self.profiles.get(&identity) == Some(&settings) {
            return Ok(());
        }
        let previous = self.profiles.insert(identity.clone(), settings);
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

    fn persist(&self) -> io::Result<()> {
        if !self.writable {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "pointer preferences were not loaded safely; refusing to overwrite the original file",
            ));
        }
        let Some(path) = self.path.as_deref() else {
            return Ok(());
        };
        let parent = path.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "pointer settings path has no parent")
        })?;
        fs::create_dir_all(parent)?;

        let document = StoredProfiles {
            format_version: FORMAT_VERSION,
            devices: self
                .profiles
                .iter()
                .map(|(identity, settings)| (identity.clone(), (*settings).into()))
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
            return Some(PathBuf::from(config_home).join("aureaos/input/pointers.toml"));
        }
    }
    std::env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(|home| PathBuf::from(home).join(".config/aureaos/input/pointers.toml"))
}

fn load_profiles(path: &Path) -> io::Result<HashMap<String, PointerSettings>> {
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
                "unsupported pointer settings format {}; expected {}",
                stored.format_version, FORMAT_VERSION
            ),
        ));
    }
    stored
        .devices
        .into_iter()
        .map(|(identity, settings)| Ok((identity, settings.try_into()?)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::PointerProfileStore;
    use aurea_desktop_api::input::{PointerAccelerationProfile, PointerSettings};
    use std::{fs, path::PathBuf};

    fn test_path(suffix: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "aurea-pointer-profiles-{}-{suffix}.toml",
            std::process::id()
        ))
    }

    #[test]
    fn profiles_survive_store_recreation_and_keep_devices_separate() {
        let path = test_path("roundtrip");
        let _ = fs::remove_file(&path);
        let mut first = PointerProfileStore::with_path(path.clone());
        let profile_a = PointerSettings::new(PointerAccelerationProfile::Flat, 0.65, false).unwrap();
        let profile_b = PointerSettings::new(PointerAccelerationProfile::Adaptive, -0.2, true).unwrap();

        first.set("physical-a".into(), profile_a).unwrap();
        first.set("physical-b".into(), profile_b).unwrap();

        let second = PointerProfileStore::with_path(path.clone());
        assert_eq!(second.get("physical-a"), Some(profile_a));
        assert_eq!(second.get("physical-b"), Some(profile_b));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn malformed_or_newer_profile_files_are_never_overwritten() {
        for (suffix, original) in [
            ("malformed", "this is not valid TOML = ["),
            ("future", "format_version = 99\n[devices]\n"),
        ] {
            let path = test_path(suffix);
            fs::write(&path, original).unwrap();
            let mut store = PointerProfileStore::with_path(path.clone());
            assert!(store
                .set("physical-a".into(), PointerSettings::default())
                .is_err());
            assert_eq!(store.get("physical-a"), None);
            assert_eq!(fs::read_to_string(&path).unwrap(), original);
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn invalid_speed_in_existing_profile_is_rejected_without_overwrite() {
        let path = test_path("invalid-speed");
        let original = "[devices.physical-a]\nprofile = \"flat\"\nspeed = 9.0\nshake_to_locate = true\n";
        fs::write(&path, original).unwrap();
        let mut store = PointerProfileStore::with_path(path.clone());
        assert!(store
            .set("physical-a".into(), PointerSettings::default())
            .is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), original);
        let _ = fs::remove_file(path);
    }
}
