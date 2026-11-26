// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Durable global desktop pointer behavior preferences.

use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use aurea_desktop_api::input::DesktopPointerBehaviorSettings;
use serde::{Deserialize, Serialize};

const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct StoredPreferences {
    format_version: u32,
    double_click_interval_ms: u64,
}

/// Stores desktop-wide pointer behavior separately from physical device profiles.
pub struct DesktopPointerBehaviorStore {
    path: Option<PathBuf>,
    settings: DesktopPointerBehaviorSettings,
    writable: bool,
}

impl Default for DesktopPointerBehaviorStore {
    fn default() -> Self {
        match default_path() {
            Some(path) => Self::open(path),
            None => Self { path: None, settings: DesktopPointerBehaviorSettings::default(), writable: true },
        }
    }
}

impl DesktopPointerBehaviorStore {
    fn open(path: PathBuf) -> Self {
        match load(&path) {
            Ok(settings) => Self { path: Some(path), settings, writable: true },
            Err(error) => {
                tracing::warn!(path = %path.display(), %error, "Desktop pointer preferences opened read-only");
                Self { path: Some(path), settings: DesktopPointerBehaviorSettings::default(), writable: false }
            }
        }
    }

    pub fn settings(&self) -> DesktopPointerBehaviorSettings {
        self.settings
    }

    /// Persists the new setting before making it the active in-memory value.
    pub fn set(&mut self, settings: DesktopPointerBehaviorSettings) -> io::Result<()> {
        self.persist(settings)?;
        self.settings = settings;
        Ok(())
    }

    fn persist(&self, settings: DesktopPointerBehaviorSettings) -> io::Result<()> {
        if !self.writable {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "refusing to overwrite unreadable desktop pointer preferences"));
        }
        let Some(path) = self.path.as_deref() else { return Ok(()); };
        let parent = path.parent().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "settings path has no parent"))?;
        fs::create_dir_all(parent)?;
        let document = StoredPreferences {
            format_version: FORMAT_VERSION,
            double_click_interval_ms: settings.double_click_interval_ms(),
        };
        let encoded = toml::to_string_pretty(&document).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
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
        if let Ok(directory) = OpenOptions::new().read(true).open(parent) { let _ = directory.sync_all(); }
        Ok(())
    }
}

fn default_path() -> Option<PathBuf> {
    if let Some(home) = std::env::var_os("XDG_CONFIG_HOME").filter(|value| !value.is_empty()) {
        return Some(PathBuf::from(home).join("aureaos/input/desktop-pointer.toml"));
    }
    std::env::var_os("HOME").filter(|value| !value.is_empty())
        .map(|home| PathBuf::from(home).join(".config/aureaos/input/desktop-pointer.toml"))
}

fn load(path: &Path) -> io::Result<DesktopPointerBehaviorSettings> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(DesktopPointerBehaviorSettings::default()),
        Err(error) => return Err(error),
    };
    let stored: StoredPreferences = toml::from_str(&content)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    if stored.format_version != FORMAT_VERSION {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "unsupported desktop pointer settings format"));
    }
    Ok(DesktopPointerBehaviorSettings::new(stored.double_click_interval_ms))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("aurea-desktop-pointer-{}-{label}.toml", std::process::id()))
    }

    #[test]
    fn preferences_survive_store_recreation() {
        let path = path("roundtrip");
        let _ = fs::remove_file(&path);
        let mut store = DesktopPointerBehaviorStore::open(path.clone());
        store.set(DesktopPointerBehaviorSettings::new(725)).unwrap();
        let reopened = DesktopPointerBehaviorStore::open(path.clone());
        assert_eq!(reopened.settings().double_click_interval_ms(), 725);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn malformed_or_newer_file_is_preserved() {
        for (suffix, original) in [("bad", "broken = ["), ("future", "format_version = 9\ndouble_click_interval_ms = 400\n")] {
            let path = path(suffix);
            fs::write(&path, original).unwrap();
            let mut store = DesktopPointerBehaviorStore::open(path.clone());
            assert!(store.set(DesktopPointerBehaviorSettings::new(500)).is_err());
            assert_eq!(fs::read_to_string(&path).unwrap(), original);
            let _ = fs::remove_file(path);
        }
    }
}
