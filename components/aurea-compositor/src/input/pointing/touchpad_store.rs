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
}

impl Default for TouchpadProfileStore {
    fn default() -> Self {
        let path = default_store_path();
        let profiles = path
            .as_deref()
            .and_then(|path| match load_profiles(path) {
                Ok(profiles) => Some(profiles),
                Err(error) => {
                    tracing::warn!(path = %path.display(), %error, "Could not load touchpad preferences");
                    None
                }
            })
            .unwrap_or_default();

        Self { path, profiles }
    }
}

impl TouchpadProfileStore {
    fn with_path(path: PathBuf) -> Self {
        let profiles = load_profiles(&path).unwrap_or_default();
        Self {
            path: Some(path),
            profiles,
        }
    }

    pub fn get(&self, identity: &str) -> Option<TouchpadSettings> {
        self.profiles.get(identity).copied()
    }

    /// Updates memory first and atomically persists the complete profile set.
    pub fn set(&mut self, identity: String, settings: TouchpadSettings) -> io::Result<()> {
        self.profiles.insert(identity, settings);
        self.persist()
    }

    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }

    fn persist(&self) -> io::Result<()> {
        let Some(path) = self.path.as_deref() else {
            return Ok(());
        };
        let parent = path.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "touchpad settings path has no parent")
        })?;
        fs::create_dir_all(parent)?;

        let document = StoredProfiles {
            format_version: 1,
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
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let content = fs::read_to_string(path)?;
    let stored: StoredProfiles = toml::from_str(&content)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    if stored.format_version > 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "touchpad settings file uses a newer unsupported format",
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

    fn test_path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "aurea-touchpad-profiles-{}.toml",
            std::process::id()
        ))
    }

    #[test]
    fn profiles_survive_store_recreation_and_keep_devices_separate() {
        let path = test_path();
        let _ = fs::remove_file(&path);
        let mut first = TouchpadProfileStore::with_path(path.clone());
        let profile_a = TouchpadSettings::default()
            .with_left_handed(true)
            .with_click_method(TouchpadClickMethod::ButtonAreas);
        let profile_b = TouchpadSettings::default()
            .with_natural_scrolling_for_test()
            .with_send_events_mode(TouchpadSendEventsMode::DisabledOnExternalMouse);

        first.set("physical-a".into(), profile_a).unwrap();
        first.set("physical-b".into(), profile_b).unwrap();

        let second = TouchpadProfileStore::with_path(path.clone());
        assert_eq!(second.get("physical-a"), Some(profile_a));
        assert_eq!(second.get("physical-b"), Some(profile_b));
        assert_eq!(second.len(), 2);
        let _ = fs::remove_file(path);
    }
}
