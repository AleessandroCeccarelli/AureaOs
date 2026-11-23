// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Safe operations for the per-user home Trash.

use std::{
    error::Error,
    fmt,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use super::metadata::{read_original_path, write_info};

/// Number of retained entries at which the icon uses its full artwork.
pub const DEFAULT_FULL_STATE_THRESHOLD: usize = 10;

/// Visual state derived from the number of entries in the Trash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrashVisualState {
    #[default]
    Empty,
    PartiallyFull,
    Full,
}

/// A file or directory currently stored in the user's Trash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrashEntry {
    name: String,
    stored_path: PathBuf,
    info_path: PathBuf,
    original_path: PathBuf,
}

impl TrashEntry {
    pub fn name(&self) -> &str { &self.name }
    pub fn original_path(&self) -> &Path { &self.original_path }
    pub fn is_directory(&self) -> bool { self.stored_path.is_dir() }
}

/// Immutable summary used by the desktop presentation model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrashSnapshot {
    item_count: usize,
    visual_state: TrashVisualState,
}

impl TrashSnapshot {
    pub fn item_count(self) -> usize { self.item_count }
    pub fn visual_state(self) -> TrashVisualState { self.visual_state }
}

/// Errors returned by Trash operations.
#[derive(Debug)]
pub enum TrashError {
    Io(std::io::Error),
    InvalidSource,
    SourceInsideTrash,
    EntryNotFound,
    RestoreDestinationExists(PathBuf),
    InvalidMetadata,
    ClockUnavailable,
}

impl fmt::Display for TrashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "Trash filesystem operation failed: {error}"),
            Self::InvalidSource => f.write_str("Trash source must be an existing absolute path"),
            Self::SourceInsideTrash => f.write_str("A Trash item cannot be moved into the Trash again"),
            Self::EntryNotFound => f.write_str("Trash entry no longer exists"),
            Self::RestoreDestinationExists(path) => write!(f, "Restore destination already exists: {}", path.display()),
            Self::InvalidMetadata => f.write_str("Trash metadata is invalid or incomplete"),
            Self::ClockUnavailable => f.write_str("System clock is earlier than the Unix epoch"),
        }
    }
}

impl Error for TrashError {}
impl From<std::io::Error> for TrashError {
    fn from(error: std::io::Error) -> Self { Self::Io(error) }
}

/// Manages the freedesktop home Trash at ~/.local/share/Trash.
///
/// This first implementation deliberately handles the home Trash only.
/// Cross-filesystem fallback Trash locations are not yet implemented.
pub struct TrashService {
    files_dir: PathBuf,
    info_dir: PathBuf,
    full_threshold: usize,
}

impl TrashService {
    /// Creates the home Trash directories below the supplied home directory.
    pub fn new(home: impl AsRef<Path>) -> Result<Self, TrashError> {
        Self::with_full_threshold(home, DEFAULT_FULL_STATE_THRESHOLD)
    }

    /// Creates the service with an explicit visual full-state threshold.
    pub fn with_full_threshold(
        home: impl AsRef<Path>,
        full_threshold: usize,
    ) -> Result<Self, TrashError> {
        let home = fs::canonicalize(home.as_ref())?;
        let root = home.join(".local/share/Trash");
        let files_dir = root.join("files");
        let info_dir = root.join("info");
        fs::create_dir_all(&files_dir)?;
        fs::create_dir_all(&info_dir)?;
        let files_dir = fs::canonicalize(files_dir)?;
        let info_dir = fs::canonicalize(info_dir)?;
        Ok(Self {
            files_dir,
            info_dir,
            full_threshold: full_threshold.max(1),
        })
    }

    /// Moves an existing absolute path into the user's Trash.
    ///
    /// The original path is recorded in a .trashinfo file before the move.
    /// A failed move removes the metadata file so no orphan entry is exposed.
    pub fn move_to_trash(&self, source: impl AsRef<Path>) -> Result<(), TrashError> {
        let source = source.as_ref();
        if !source.is_absolute() || !source.exists() {
            return Err(TrashError::InvalidSource);
        }
        let canonical = fs::canonicalize(source)?;
        if canonical.starts_with(&self.files_dir) || canonical.starts_with(&self.info_dir) {
            return Err(TrashError::SourceInsideTrash);
        }

        let original_name = source.file_name().ok_or(TrashError::InvalidSource)?;
        let stored_name = self.unique_name(original_name)?;
        let stored_path = self.files_dir.join(&stored_name);
        let info_path = self.info_dir.join(format!("{stored_name}.trashinfo"));
        let deletion_date = deletion_timestamp()?;
        fs::write(&info_path, write_info(source, &deletion_date))?;

        if let Err(error) = fs::rename(source, &stored_path) {
            let _ = fs::remove_file(&info_path);
            return Err(TrashError::Io(error));
        }
        Ok(())
    }

    /// Lists valid entries whose data and metadata files both exist.
    pub fn entries(&self) -> Result<Vec<TrashEntry>, TrashError> {
        let mut entries = Vec::new();
        for item in fs::read_dir(&self.files_dir)? {
            let item = item?;
            let stored_path = item.path();
            let name = item.file_name().to_string_lossy().into_owned();
            let info_path = self.info_dir.join(format!("{name}.trashinfo"));
            if !info_path.is_file() {
                continue;
            }
            let info = fs::read_to_string(&info_path)?;
            let original_path = read_original_path(&info).ok_or(TrashError::InvalidMetadata)?;
            if !original_path.is_absolute() {
                return Err(TrashError::InvalidMetadata);
            }
            entries.push(TrashEntry {
                name,
                stored_path,
                info_path,
                original_path,
            });
        }
        entries.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(entries)
    }

    /// Returns the current item count and visual state.
    pub fn snapshot(&self) -> Result<TrashSnapshot, TrashError> {
        let item_count = self.entries()?.len();
        let visual_state = if item_count == 0 {
            TrashVisualState::Empty
        } else if item_count >= self.full_threshold {
            TrashVisualState::Full
        } else {
            TrashVisualState::PartiallyFull
        };
        Ok(TrashSnapshot { item_count, visual_state })
    }

    /// Restores an entry to its recorded original location.
    ///
    /// Existing destinations are never overwritten.
    pub fn restore(&self, entry_name: &str) -> Result<PathBuf, TrashError> {
        let entry = self.find_entry(entry_name)?;
        if entry.original_path.exists() {
            return Err(TrashError::RestoreDestinationExists(entry.original_path));
        }
        if let Some(parent) = entry.original_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(&entry.stored_path, &entry.original_path)?;
        fs::remove_file(&entry.info_path)?;
        Ok(entry.original_path)
    }

    /// Permanently removes one entry. This operation is intentionally explicit.
    pub fn delete_permanently(&self, entry_name: &str) -> Result<(), TrashError> {
        let entry = self.find_entry(entry_name)?;
        if entry.stored_path.is_dir() {
            fs::remove_dir_all(&entry.stored_path)?;
        } else {
            fs::remove_file(&entry.stored_path)?;
        }
        fs::remove_file(&entry.info_path)?;
        Ok(())
    }

    /// Permanently empties all valid and orphaned entries in the home Trash.
    pub fn empty(&self) -> Result<(), TrashError> {
        for item in fs::read_dir(&self.files_dir)? {
            let path = item?.path();
            if path.is_dir() {
                fs::remove_dir_all(path)?;
            } else {
                fs::remove_file(path)?;
            }
        }
        for item in fs::read_dir(&self.info_dir)? {
            let path = item?.path();
            if path.is_file() {
                fs::remove_file(path)?;
            }
        }
        Ok(())
    }

    fn find_entry(&self, name: &str) -> Result<TrashEntry, TrashError> {
        self.entries()?
            .into_iter()
            .find(|entry| entry.name == name)
            .ok_or(TrashError::EntryNotFound)
    }

    fn unique_name(&self, original_name: &std::ffi::OsStr) -> Result<String, TrashError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| TrashError::ClockUnavailable)?
            .as_nanos();
        let base = original_name.to_string_lossy();
        let candidate = format!("{timestamp}-{base}");
        if !self.files_dir.join(&candidate).exists()
            && !self.info_dir.join(format!("{candidate}.trashinfo")).exists()
        {
            return Ok(candidate);
        }
        for suffix in 1..=u32::MAX {
            let candidate = format!("{timestamp}-{suffix}-{base}");
            if !self.files_dir.join(&candidate).exists()
                && !self.info_dir.join(format!("{candidate}.trashinfo")).exists()
            {
                return Ok(candidate);
            }
        }
        Err(TrashError::EntryNotFound)
    }
}

fn deletion_timestamp() -> Result<String, TrashError> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| TrashError::ClockUnavailable)?
        .as_secs();
    let days = seconds / 86_400;
    let seconds_of_day = seconds % 86_400;
    let (year, month, day) = civil_from_days(days as i64);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    Ok(format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}"))
}

fn civil_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::{TrashError, TrashService, TrashVisualState};
    use std::{fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

    struct TestHome(PathBuf);
    impl TestHome {
        fn new() -> Self {
            let id = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let path = std::env::temp_dir().join(format!("aurea-trash-test-{id}"));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for TestHome {
        fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
    }

    #[test]
    fn moves_lists_restores_and_updates_visual_state() {
        let home = TestHome::new();
        let source = home.0.join("report.txt");
        fs::write(&source, "hello").unwrap();
        let trash = TrashService::new(&home.0).unwrap();
        assert_eq!(trash.snapshot().unwrap().visual_state(), TrashVisualState::Empty);

        trash.move_to_trash(&source).unwrap();
        assert!(!source.exists());
        let entries = trash.entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(trash.snapshot().unwrap().visual_state(), TrashVisualState::PartiallyFull);

        let restored = trash.restore(entries[0].name()).unwrap();
        assert_eq!(restored, source);
        assert_eq!(fs::read_to_string(restored).unwrap(), "hello");
        assert_eq!(trash.snapshot().unwrap().visual_state(), TrashVisualState::Empty);
    }

    #[test]
    fn refuses_to_overwrite_restore_destination() {
        let home = TestHome::new();
        let source = home.0.join("report.txt");
        fs::write(&source, "old").unwrap();
        let trash = TrashService::new(&home.0).unwrap();
        trash.move_to_trash(&source).unwrap();
        fs::write(&source, "new").unwrap();
        let entry = trash.entries().unwrap().remove(0);
        assert!(matches!(trash.restore(entry.name()), Err(TrashError::RestoreDestinationExists(_))));
        assert_eq!(fs::read_to_string(source).unwrap(), "new");
    }

    #[test]
    fn full_state_uses_configured_threshold() {
        let home = TestHome::new();
        let trash = TrashService::with_full_threshold(&home.0, 1).unwrap();
        let source = home.0.join("item");
        fs::write(&source, "x").unwrap();
        trash.move_to_trash(&source).unwrap();
        assert_eq!(trash.snapshot().unwrap().visual_state(), TrashVisualState::Full);
    }

    #[test]
    fn permanently_deletes_only_the_selected_entry() {
        let home = TestHome::new();
        let source = home.0.join("report.txt");
        fs::write(&source, "hello").unwrap();
        let trash = TrashService::new(&home.0).unwrap();
        trash.move_to_trash(&source).unwrap();
        let entry = trash.entries().unwrap().remove(0);
        trash.delete_permanently(entry.name()).unwrap();
        assert!(trash.entries().unwrap().is_empty());
    }
}
