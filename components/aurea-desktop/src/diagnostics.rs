// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Persistent diagnostics for native desktop test runs.

use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    panic,
    path::PathBuf,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

static LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();

/// Opens the per-user log and installs a panic hook that preserves crash details.
pub fn initialize() -> io::Result<PathBuf> {
    let directory = resolve_state_directory(env::var_os("XDG_STATE_HOME"), env::var_os("HOME"))?
        .join("aureaos")
        .join("logs");
    fs::create_dir_all(&directory)?;
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))?;

    let path = directory.join("aurea-desktop.log");
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .open(&path)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    LOG_FILE
        .set(Mutex::new(file))
        .map_err(|_| io::Error::other("desktop diagnostics already initialized"))?;

    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(move |information| {
        record("PANIC", &information.to_string());
        previous_hook(information);
    }));

    record("INFO", "Persistent diagnostics initialized");
    Ok(path)
}

/// Appends one timestamped diagnostic entry and flushes it to storage.
pub fn record(level: &str, message: &str) {
    let Some(file) = LOG_FILE.get() else {
        eprintln!("Aurea Desktop [{level}]: {message}");
        return;
    };

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();

    if let Ok(mut file) = file.lock() {
        let _ = writeln!(file, "{timestamp} [{level}] {message}");
        let _ = file.flush();
        let _ = file.sync_data();
    }
}

fn resolve_state_directory(
    xdg_state_home: Option<std::ffi::OsString>,
    home: Option<std::ffi::OsString>,
) -> io::Result<PathBuf> {
    if let Some(path) = xdg_state_home.filter(|value| !value.is_empty()) {
        return Ok(PathBuf::from(path));
    }

    let home = home.filter(|value| !value.is_empty()).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "HOME and XDG_STATE_HOME are unset")
    })?;

    Ok(PathBuf::from(home).join(".local").join("state"))
}

#[cfg(test)]
mod tests {
    use super::resolve_state_directory;
    use std::{ffi::OsString, path::PathBuf};

    #[test]
    fn prefers_xdg_state_home_when_available() {
        let path = resolve_state_directory(
            Some(OsString::from("/tmp/aurea-state")),
            Some(OsString::from("/home/tester")),
        )
        .unwrap();

        assert_eq!(path, PathBuf::from("/tmp/aurea-state"));
    }

    #[test]
    fn falls_back_to_local_state_under_home() {
        let path = resolve_state_directory(None, Some(OsString::from("/home/tester"))).unwrap();

        assert_eq!(path, PathBuf::from("/home/tester/.local/state"));
    }

    #[test]
    fn reports_missing_state_directory_environment() {
        assert!(resolve_state_directory(None, None).is_err());
    }
}
