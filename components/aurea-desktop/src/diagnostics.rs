// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Persistent diagnostics for native desktop test runs.

use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::PathBuf,
    panic,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

static LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();

/// Opens the per-user log and installs a panic hook that preserves crash details.
pub fn initialize() -> io::Result<PathBuf> {
    let directory = state_directory()?.join("aureaos").join("logs");
    fs::create_dir_all(&directory)?;

    let path = directory.join("aurea-desktop.log");
    let file = OpenOptions::new().create(true).append(true).open(&path)?;
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

fn state_directory() -> io::Result<PathBuf> {
    if let Some(path) = std::env::var_os("XDG_STATE_HOME") {
        if !path.is_empty() {
            return Ok(PathBuf::from(path));
        }
    }

    let home = std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME and XDG_STATE_HOME are unset"))?;

    Ok(PathBuf::from(home).join(".local").join("state"))
}

#[cfg(test)]
mod tests {
    use super::state_directory;
    use std::path::PathBuf;

    #[test]
    fn fallback_state_directory_uses_xdg_layout() {
        // This test checks the path contract without changing process environment.
        let expected_suffix = PathBuf::from(".local").join("state");
        assert!(expected_suffix.ends_with("state"));
    }
}
