// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Persistent user selection for the session keyboard layout.

use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::PathBuf,
};

use aurea_desktop_api::keyboard::KeyboardLayoutId;

/// Reads and atomically persists the user's keyboard layout preference.
pub struct KeyboardLayoutStore {
    path: PathBuf,
}

impl KeyboardLayoutStore {
    /// Resolves the per-user configuration path.
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

        Ok(Self {
            path: config_home.join("aureaos").join("keyboard.conf"),
        })
    }

    /// Loads a saved selection. Missing files mean that XKB defaults remain active.
    pub fn load(&self) -> io::Result<Option<KeyboardLayoutId>> {
        let content = match fs::read_to_string(&self.path) {
            Ok(content) => content,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };

        parse_layout(&content).map(Some).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "invalid keyboard layout configuration")
        })
    }

    /// Saves the selected layout using a same-directory atomic rename.
    pub fn save(&self, layout: &KeyboardLayoutId) -> io::Result<()> {
        let parent = self.path.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "keyboard config path has no parent")
        })?;
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
        let variant = layout.variant().unwrap_or("");
        let result = writeln!(file, "layout={}\nvariant={variant}", layout.layout())
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

fn parse_layout(content: &str) -> Option<KeyboardLayoutId> {
    let mut layout = None;
    let mut variant = None;

    for line in content.lines() {
        let (key, value) = line.split_once('=')?;
        match key {
            "layout" if layout.is_none() => layout = Some(value.to_owned()),
            "variant" if variant.is_none() => {
                variant = (!value.is_empty()).then(|| value.to_owned());
            }
            _ => return None,
        }
    }

    KeyboardLayoutId::new(layout?, variant)
}

#[cfg(test)]
mod tests {
    use super::parse_layout;

    #[test]
    fn parses_layout_and_optional_variant() {
        let layout = parse_layout("layout=us\nvariant=intl\n").expect("valid config");
        assert_eq!(layout.layout(), "us");
        assert_eq!(layout.variant(), Some("intl"));
    }

    #[test]
    fn rejects_unknown_keys_and_invalid_identifiers() {
        assert!(parse_layout("layout=it;touch\n").is_none());
        assert!(parse_layout("layout=it\nextra=value\n").is_none());
    }
}
