// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Atomic persistence for the user's still-image wallpaper preferences.

use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::PathBuf,
};

use aurea_desktop_api::desktop::{WallpaperPreferences, WallpaperScaleMode};

/// Stores wallpaper selection and fit mode in the per-user AureaOS config.
pub struct WallpaperPreferencesStore {
    path: PathBuf,
}

impl WallpaperPreferencesStore {
    pub fn new() -> io::Result<Self> {
        let config_home = match std::env::var_os("XDG_CONFIG_HOME") {
            Some(path) if !path.is_empty() => PathBuf::from(path),
            _ => {
                let home = std::env::var_os("HOME")
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| io::Error::new(
                        io::ErrorKind::NotFound,
                        "neither XDG_CONFIG_HOME nor HOME is set",
                    ))?;
                PathBuf::from(home).join(".config")
            }
        };
        Ok(Self { path: config_home.join("aureaos").join("wallpaper.conf") })
    }

    /// Loads preferences; a missing file means default wallpaper and Fill mode.
    pub fn load(&self) -> io::Result<WallpaperPreferences> {
        let content = match fs::read_to_string(&self.path) {
            Ok(content) => content,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(WallpaperPreferences::default());
            }
            Err(error) => return Err(error),
        };
        parse_preferences(&content).ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid wallpaper configuration",
        ))
    }

    /// Persists preferences with a same-directory atomic rename and mode 0600.
    pub fn save(&self, preferences: &WallpaperPreferences) -> io::Result<()> {
        let parent = self.path.parent().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidInput,
            "wallpaper config path has no parent",
        ))?;
        fs::create_dir_all(parent)?;
        let temporary = self.path.with_extension(format!("conf.{}.tmp", std::process::id()));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temporary)?;
        let path = preferences.image_path().unwrap_or("");
        let mode = match preferences.scale_mode() {
            WallpaperScaleMode::Fill => "fill",
            WallpaperScaleMode::Fit => "fit",
            WallpaperScaleMode::Stretch => "stretch",
            WallpaperScaleMode::Center => "center",
        };
        let escaped_path = path.replace('\\', "\\\\").replace('\n', "\\n");
        let result = writeln!(file, "path={escaped_path}\nmode={mode}")
            .and_then(|()| file.sync_all());
        if let Err(error) = result {
            let _ = fs::remove_file(&temporary);
            return Err(error);
        }
        drop(file);
        if let Err(error) = fs::rename(&temporary, &self.path) {
            let _ = fs::remove_file(&temporary);
            return Err(error);
        }
        Ok(())
    }
}

fn parse_preferences(content: &str) -> Option<WallpaperPreferences> {
    let mut path = None;
    let mut mode = None;
    for line in content.lines() {
        let (key, value) = line.split_once('=')?;
        match key {
            "path" if path.is_none() => {
                let decoded = value.replace("\\n", "\n").replace("\\\\", "\\");
                path = Some(decoded);
            }
            "mode" if mode.is_none() => {
                mode = Some(match value {
                    "fill" => WallpaperScaleMode::Fill,
                    "fit" => WallpaperScaleMode::Fit,
                    "stretch" => WallpaperScaleMode::Stretch,
                    "center" => WallpaperScaleMode::Center,
                    _ => return None,
                });
            }
            _ => return None,
        }
    }
    let mut preferences = WallpaperPreferences::default();
    preferences.set_image_path(path?);
    preferences.set_scale_mode(mode?);
    Some(preferences)
}

#[cfg(test)]
mod tests {
    use super::parse_preferences;
    use aurea_desktop_api::desktop::{WallpaperScaleMode};

    #[test]
    fn round_trip_parser_accepts_paths_and_modes() {
        let preferences = parse_preferences("path=/home/user/My\\nWallpapers/image.jpg\nmode=fit\n").unwrap();
        assert_eq!(preferences.image_path(), Some("/home/user/My\nWallpapers/image.jpg"));
        assert_eq!(preferences.scale_mode(), WallpaperScaleMode::Fit);
    }

    #[test]
    fn rejects_unknown_keys_and_modes() {
        assert!(parse_preferences("path=/tmp/a.jpg\nmode=video\n").is_none());
        assert!(parse_preferences("path=/tmp/a.jpg\nmode=fill\nextra=x\n").is_none());
    }
}
