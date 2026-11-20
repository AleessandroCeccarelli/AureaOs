// SPDX-License-Identifier: GPL-3.0-or-later
// AureaOS — Color Management

//! Bounded, content-addressed storage for validated ICC profile files.

use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    IccProfileFingerprint, IccProfileSummary, MAX_PROFILE_SIZE, fingerprint_profile,
    inspect_profile,
};

static TEMPORARY_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Failures returned by the local ICC profile store.
#[derive(Debug)]
pub enum ProfileStoreError {
    InvalidProfile,
    ProfileTooLarge,
    ProfileNotFound,
    UnsafeStorageEntry,
    Io(std::io::Error),
}

impl From<std::io::Error> for ProfileStoreError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// Metadata returned after a profile has been stored or loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredProfile {
    pub fingerprint: IccProfileFingerprint,
    pub summary: IccProfileSummary,
}

/// Local content-addressed profile store.
///
/// The store owns only files named from validated SHA-256 fingerprints.
/// User-supplied names and paths are never used to construct storage paths.
pub struct IccProfileStore {
    root: PathBuf,
}

impl IccProfileStore {
    /// Opens or creates a dedicated profile directory.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, ProfileStoreError> {
        let requested_root = root.as_ref();
        fs::create_dir_all(requested_root)?;
        let metadata = fs::symlink_metadata(requested_root)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(ProfileStoreError::UnsafeStorageEntry);
        }

        let root = fs::canonicalize(requested_root)?;
        Ok(Self { root })
    }

    /// Validates and stores a profile atomically, deduplicating identical bytes.
    pub fn store(&self, bytes: &[u8]) -> Result<StoredProfile, ProfileStoreError> {
        if bytes.len() > MAX_PROFILE_SIZE {
            return Err(ProfileStoreError::ProfileTooLarge);
        }

        let summary = inspect_profile(bytes).map_err(|_| ProfileStoreError::InvalidProfile)?;
        let fingerprint =
            fingerprint_profile(bytes).map_err(|_| ProfileStoreError::InvalidProfile)?;
        let destination = self.profile_path(&fingerprint);

        if destination.exists() {
            let existing = self.load(&fingerprint)?;
            return Ok(existing);
        }

        let temporary = self.temporary_path();
        let write_result = self.write_temporary(&temporary, bytes);
        if let Err(error) = write_result {
            let _ = fs::remove_file(&temporary);
            return Err(error);
        }

        if let Err(error) = fs::rename(&temporary, &destination) {
            let _ = fs::remove_file(&temporary);
            if destination.exists() {
                return self.load(&fingerprint);
            }
            return Err(ProfileStoreError::Io(error));
        }

        if let Ok(directory) = File::open(&self.root) {
            directory.sync_all()?;
        }

        Ok(StoredProfile {
            fingerprint,
            summary,
        })
    }

    /// Loads and revalidates a profile by its content fingerprint.
    pub fn load(
        &self,
        fingerprint: &IccProfileFingerprint,
    ) -> Result<StoredProfile, ProfileStoreError> {
        let bytes = self.read_bytes(fingerprint)?;
        let summary = inspect_profile(&bytes).map_err(|_| ProfileStoreError::InvalidProfile)?;
        Ok(StoredProfile {
            fingerprint: fingerprint.clone(),
            summary,
        })
    }

    /// Returns the exact stored bytes after validating their structure and identity.
    pub fn read_bytes(
        &self,
        fingerprint: &IccProfileFingerprint,
    ) -> Result<Vec<u8>, ProfileStoreError> {
        let path = self.profile_path(fingerprint);
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                ProfileStoreError::ProfileNotFound
            } else {
                ProfileStoreError::Io(error)
            }
        })?;

        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(ProfileStoreError::UnsafeStorageEntry);
        }
        if metadata.len() > MAX_PROFILE_SIZE as u64 {
            return Err(ProfileStoreError::ProfileTooLarge);
        }

        let mut file = File::open(path)?;
        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        file.take((MAX_PROFILE_SIZE + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_PROFILE_SIZE {
            return Err(ProfileStoreError::ProfileTooLarge);
        }

        let actual_fingerprint =
            fingerprint_profile(&bytes).map_err(|_| ProfileStoreError::InvalidProfile)?;
        if &actual_fingerprint != fingerprint {
            return Err(ProfileStoreError::InvalidProfile);
        }
        Ok(bytes)
    }

    /// Removes a stored profile. Unknown fingerprints are reported explicitly.
    pub fn remove(&self, fingerprint: &IccProfileFingerprint) -> Result<(), ProfileStoreError> {
        let path = self.profile_path(fingerprint);
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                ProfileStoreError::ProfileNotFound
            } else {
                ProfileStoreError::Io(error)
            }
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(ProfileStoreError::UnsafeStorageEntry);
        }
        fs::remove_file(path)?;
        Ok(())
    }

    fn profile_path(&self, fingerprint: &IccProfileFingerprint) -> PathBuf {
        self.root.join(format!("{}.icc", fingerprint.to_hex()))
    }

    fn temporary_path(&self) -> PathBuf {
        let sequence = TEMPORARY_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        self.root
            .join(format!(".profile-{}-{sequence}.tmp", std::process::id()))
    }

    fn write_temporary(&self, path: &Path, bytes: &[u8]) -> Result<(), ProfileStoreError> {
        let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::{IccProfileStore, ProfileStoreError};

    static TEST_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn temporary_directory() -> std::path::PathBuf {
        let sequence = TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "aurea-color-store-{}-{sequence}",
            std::process::id()
        ))
    }

    fn minimal_profile() -> Vec<u8> {
        let mut bytes = vec![0_u8; 132];
        bytes[0..4].copy_from_slice(&(132_u32).to_be_bytes());
        bytes[8..12].copy_from_slice(b"\x04\x30\x00\x00");
        bytes[12..16].copy_from_slice(b"mntr");
        bytes[16..20].copy_from_slice(b"RGB ");
        bytes[20..24].copy_from_slice(b"XYZ ");
        bytes[36..40].copy_from_slice(b"acsp");
        bytes
    }

    #[test]
    fn stores_and_reloads_profile_without_changing_identity() {
        let root = temporary_directory();
        let store = IccProfileStore::open(&root).unwrap();
        let stored = store.store(&minimal_profile()).unwrap();
        let loaded = store.load(&stored.fingerprint).unwrap();

        assert_eq!(stored, loaded);
        store.remove(&stored.fingerprint).unwrap();
        assert!(matches!(
            store.load(&stored.fingerprint),
            Err(ProfileStoreError::ProfileNotFound)
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn identical_profiles_are_deduplicated() {
        let root = temporary_directory();
        let store = IccProfileStore::open(&root).unwrap();
        let first = store.store(&minimal_profile()).unwrap();
        let second = store.store(&minimal_profile()).unwrap();

        assert_eq!(first.fingerprint, second.fingerprint);
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn malformed_profiles_are_never_persisted() {
        let root = temporary_directory();
        let store = IccProfileStore::open(&root).unwrap();
        assert!(matches!(
            store.store(b"not an ICC profile"),
            Err(ProfileStoreError::InvalidProfile)
        ));
        assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
        fs::remove_dir_all(root).unwrap();
    }
}
