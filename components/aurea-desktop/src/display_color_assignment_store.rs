// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Durable storage for per-display color configuration assignments.

use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use aurea_desktop_api::display::{
    DisplayColorConfigurationAssignment, DisplayColorConfigurationId, DisplayId,
};

const MAX_ASSIGNMENT_FILE_BYTES: u64 = 1024 * 1024;
static TEMPORARY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub enum DisplayColorStoreError {
    InvalidData,
    UnsafePath,
    Io(std::io::Error),
}

impl From<std::io::Error> for DisplayColorStoreError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// Stores assignments independently from ICC profile payloads.
///
/// The file contains runtime DisplayIds and opaque configuration IDs.
/// The caller must only restore an assignment after resolving that ID against
/// the current trusted display inventory. DisplayIds are not, by themselves,
/// proof of a persistent physical-monitor identity. Profile bytes remain in
/// the ICC profile store.
#[derive(Debug, Clone)]
pub struct DisplayColorAssignmentStore {
    path: PathBuf,
}

impl DisplayColorAssignmentStore {
    /// Opens the assignment file, creating its parent directory when needed.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DisplayColorStoreError> {
        let path = path.as_ref();
        let parent = path.parent().ok_or(DisplayColorStoreError::UnsafePath)?;
        fs::create_dir_all(parent)?;
        let parent_metadata = fs::symlink_metadata(parent)?;
        if parent_metadata.file_type().is_symlink() || !parent_metadata.is_dir() {
            return Err(DisplayColorStoreError::UnsafePath);
        }
        let canonical_parent = fs::canonicalize(parent)?;
        let file_name = path.file_name().ok_or(DisplayColorStoreError::UnsafePath)?;
        let path = canonical_parent.join(file_name);

        if let Ok(metadata) = fs::symlink_metadata(&path) {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(DisplayColorStoreError::UnsafePath);
            }
            if metadata.len() > MAX_ASSIGNMENT_FILE_BYTES {
                return Err(DisplayColorStoreError::InvalidData);
            }
        }

        Ok(Self { path })
    }

    /// Resolves the per-user AureaOS configuration location.
    pub fn user_default() -> Result<Self, DisplayColorStoreError> {
        let root = if let Some(path) = std::env::var_os("XDG_CONFIG_HOME") {
            PathBuf::from(path)
        } else {
            let home = std::env::var_os("HOME").ok_or(DisplayColorStoreError::UnsafePath)?;
            PathBuf::from(home).join(".config")
        };
        Self::open(root.join("aurea/color/display-assignments"))
    }

    /// Loads saved assignments. A missing file is an empty configuration.
    pub fn load(&self) -> Result<Vec<DisplayColorConfigurationAssignment>, DisplayColorStoreError> {
        let metadata = match fs::symlink_metadata(&self.path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(DisplayColorStoreError::Io(error)),
        };
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(DisplayColorStoreError::UnsafePath);
        }
        if metadata.len() > MAX_ASSIGNMENT_FILE_BYTES {
            return Err(DisplayColorStoreError::InvalidData);
        }

        let mut file = File::open(&self.path)?;
        let mut contents = String::new();
        file.take(MAX_ASSIGNMENT_FILE_BYTES + 1)
            .read_to_string(&mut contents)?;
        if contents.len() as u64 > MAX_ASSIGNMENT_FILE_BYTES {
            return Err(DisplayColorStoreError::InvalidData);
        }

        let mut assignments = Vec::new();
        for line in contents.lines() {
            let (display, encoded_id) = line
                .split_once('\t')
                .ok_or(DisplayColorStoreError::InvalidData)?;
            let display_id = display
                .parse::<u128>()
                .map_err(|_| DisplayColorStoreError::InvalidData)?;
            let bytes = decode_hex(encoded_id)?;
            let value =
                String::from_utf8(bytes).map_err(|_| DisplayColorStoreError::InvalidData)?;
            let configuration_id = DisplayColorConfigurationId::new(value)
                .ok_or(DisplayColorStoreError::InvalidData)?;
            if assignments
                .iter()
                .any(|entry: &DisplayColorConfigurationAssignment| {
                    entry.display_id == DisplayId(display_id)
                })
            {
                return Err(DisplayColorStoreError::InvalidData);
            }
            assignments.push(DisplayColorConfigurationAssignment {
                display_id: DisplayId(display_id),
                configuration_id: Some(configuration_id),
            });
        }
        Ok(assignments)
    }

    /// Atomically replaces the complete assignment snapshot.
    pub fn save(
        &self,
        assignments: &[DisplayColorConfigurationAssignment],
    ) -> Result<(), DisplayColorStoreError> {
        let mut ordered = assignments.to_vec();
        ordered.sort_by_key(|assignment| assignment.display_id);
        let mut contents = String::new();
        for assignment in ordered {
            if let Some(configuration_id) = assignment.configuration_id {
                contents.push_str(&assignment.display_id.0.to_string());
                contents.push('\t');
                contents.push_str(&encode_hex(configuration_id.as_str().as_bytes()));
                contents.push('\n');
            }
        }
        if contents.len() as u64 > MAX_ASSIGNMENT_FILE_BYTES {
            return Err(DisplayColorStoreError::InvalidData);
        }

        if let Ok(metadata) = fs::symlink_metadata(&self.path) {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(DisplayColorStoreError::UnsafePath);
            }
        }

        let sequence = TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temporary = self
            .path
            .with_extension(format!("tmp-{}-{sequence}", std::process::id()));
        let result = (|| {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)?;
            file.write_all(contents.as_bytes())?;
            file.sync_all()?;
            fs::rename(&temporary, &self.path)?;
            if let Ok(directory) = File::open(
                self.path
                    .parent()
                    .ok_or(DisplayColorStoreError::UnsafePath)?,
            ) {
                directory.sync_all()?;
            }
            Ok::<(), DisplayColorStoreError>(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn decode_hex(value: &str) -> Result<Vec<u8>, DisplayColorStoreError> {
    if value.is_empty() || value.len() % 2 != 0 {
        return Err(DisplayColorStoreError::InvalidData);
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = (pair[0] as char)
                .to_digit(16)
                .ok_or(DisplayColorStoreError::InvalidData)?;
            let low = (pair[1] as char)
                .to_digit(16)
                .ok_or(DisplayColorStoreError::InvalidData)?;
            Ok(((high << 4) | low) as u8)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{DisplayColorAssignmentStore, DisplayColorStoreError};
    use aurea_desktop_api::display::{
        DisplayColorConfigurationAssignment, DisplayColorConfigurationId, DisplayId,
    };
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };

    static SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn test_path() -> std::path::PathBuf {
        let id = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir()
            .join(format!("aurea-display-color-{}-{id}", std::process::id()))
            .join("assignments")
    }

    #[test]
    fn persists_and_restores_assignments() {
        let path = test_path();
        let store = DisplayColorAssignmentStore::open(&path).unwrap();
        let assignments = vec![DisplayColorConfigurationAssignment {
            display_id: DisplayId(42),
            configuration_id: Some(DisplayColorConfigurationId::new("studio:calibrated").unwrap()),
        }];
        store.save(&assignments).unwrap();
        assert_eq!(store.load().unwrap(), assignments);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn missing_file_is_an_empty_snapshot() {
        let path = test_path();
        let store = DisplayColorAssignmentStore::open(&path).unwrap();
        assert!(store.load().unwrap().is_empty());
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn rejects_duplicate_display_records() {
        let path = test_path();
        let store = DisplayColorAssignmentStore::open(&path).unwrap();
        fs::write(&path, "7\t61\n7\t62\n").unwrap();
        assert!(matches!(
            store.load(),
            Err(DisplayColorStoreError::InvalidData)
        ));
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
