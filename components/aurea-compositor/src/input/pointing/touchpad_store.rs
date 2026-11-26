// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Durable touchpad preferences keyed by privacy-preserving physical identity.

use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use aurea_desktop_api::input::TouchpadSettings;
use serde::{Deserialize, Serialize};

const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct StoredProfiles {
    format_version: u32,
    devices: HashMap<String, TouchpadSettings>,
}

/// Stores user preferences independently from transient libinput device IDs.
pub struct TouchpadProfileStore {
    path: Option<PathBuf>,
    profiles: HashMap<String, TouchpadSettings>,
    writable: bool,
}

impl Default for TouchpadProfileStore {
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

impl TouchpadProfileStore {
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
                // Preserve unreadable, malformed and newer-version files. A
                // later preference change must never replace unknown user data.
                tracing::warn!(
                    path = %path.display(),
                    %error,
                    "Touchpad preferences opened read-only; original file will be preserved"
                );
                Self {
                    path: Some(path),
                    profiles: HashMap::new(),
                    writable: false,
                }
            }
        }
    }

    pub fn get(&self, identity: &str) -> Option<TouchpadSettings> {
        self.profiles.get(identity).copied()
    }

    /// Updates the in-memory profile and persists it when the store is writable.
    pub fn set(&mut self, identity: String, settings: TouchpadSettings) -> io::Result<()> {
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

    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }

    fn persist(&self) -> io::Result<()> {
        if !self.writable {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "touchpad preferences were not loaded safely; refusing to overwrite the original file",
            ));
        }
        let Some(path) = self.path.as_deref() else {
            return Ok(());
        };
        let parent = path.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "touchpad settings path has no parent")
        })?;
        fs::create_dir_all(parent)?;

        let document = StoredProfiles {
            format_version: FORMAT_VERSION,
            devices: self.profiles.clone(),
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
            return Some(PathBuf::from(config_home).join("aureaos/input/touchpads.toml"));
        }
    }
    std::env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(|home| PathBuf::from(home).join(".config/aureaos/input/touchpads.toml"))
}

fn load_profiles(path: &Path) -> io::Result<HashMap<String, TouchpadSettings>> {
    // Treat only an actual NotFound result as a first-run configuration.
    // Path::exists() suppresses metadata errors, which could otherwise make
    // an unreadable existing file look like an empty profile store.
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(HashMap::new());
        }
        Err(error) => return Err(error),
    };
    let stored: StoredProfiles = toml::from_str(&content)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    if stored.format_version != FORMAT_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "unsupported touchpad settings format {}; expected {}",
                stored.format_version, FORMAT_VERSION
            ),
        ));
    }
    Ok(stored.devices)
}

#[cfg(test)]
mod tests {
    use super::TouchpadProfileStore;
    use aurea_desktop_api::input::{
        TouchpadClickMethod, TouchpadSendEventsMode, TouchpadSettings,
    };
    use std::{fs, path::PathBuf};

    fn test_path(suffix: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "aurea-touchpad-profiles-{}-{suffix}.toml",
            std::process::id()
        ))
    }

    #[test]
    fn profiles_survive_store_recreation_and_keep_devices_separate() {
        let path = test_path("roundtrip");
        let _ = fs::remove_file(&path);
        let mut first = TouchpadProfileStore::with_path(path.clone());
        let profile_a = TouchpadSettings::default()
            .with_left_handed(true)
            .with_click_method(TouchpadClickMethod::ButtonAreas);
        let profile_b = TouchpadSettings::new(true, true, true, true, true)
            .with_send_events_mode(TouchpadSendEventsMode::DisabledOnExternalMouse);

        first.set("physical-a".into(), profile_a).unwrap();
        first.set("physical-b".into(), profile_b).unwrap();

        let second = TouchpadProfileStore::with_path(path.clone());
        assert_eq!(second.get("physical-a"), Some(profile_a));
        assert_eq!(second.get("physical-b"), Some(profile_b));
        assert_eq!(second.len(), 2);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn malformed_profile_file_is_never_overwritten() {
        let path = test_path("malformed");
        let original = "this is not valid TOML = [";
        fs::write(&path, original).unwrap();

        let mut store = TouchpadProfileStore::with_path(path.clone());
        assert!(store.set("physical-a".into(), TouchpadSettings::default()).is_err());
        assert_eq!(store.get("physical-a"), None);
        assert_eq!(fs::read_to_string(&path).unwrap(), original);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn newer_profile_format_is_never_overwritten() {
        let path = test_path("future");
        let original = "format_version = 99\n[devices]\n";
        fs::write(&path, original).unwrap();

        let mut store = TouchpadProfileStore::with_path(path.clone());
        assert!(store.set("physical-a".into(), TouchpadSettings::default()).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), original);
        let _ = fs::remove_file(path);
    }
}
