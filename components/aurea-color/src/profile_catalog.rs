// SPDX-License-Identifier: GPL-3.0-or-later
// AureaOS — Color Management

//! Concrete ICC catalog coordinator.
//!
//! Profile bytes remain in the content-addressed store. This module owns
//! descriptors, per-display assignments and change notifications.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::{
        Mutex,
        mpsc::{self, Receiver, Sender},
    },
};

use aurea_desktop_api::display::{
    DisplayIccProfileAssignment, DisplayId, IccProfileCatalogSnapshot, IccProfileClass,
    IccProfileDescriptor, IccProfileFormat, IccProfileId, IccProfileImport, IccProfileManager,
    IccProfileManagerError, IccProfileOrigin, IccProfileProvider, MAX_ICC_PROFILE_BYTES,
};

use crate::{IccProfileFingerprint, IccProfileStore, ProfileStoreError};

struct CatalogEntry {
    descriptor: IccProfileDescriptor,
    fingerprint: IccProfileFingerprint,
}

#[derive(Default)]
struct CatalogState {
    profiles: BTreeMap<String, CatalogEntry>,
    assignments: BTreeMap<DisplayId, IccProfileId>,
    connected_displays: BTreeSet<DisplayId>,
    subscribers: Vec<Sender<IccProfileCatalogSnapshot>>,
}

/// Coordinates validated ICC profiles, output assignments and catalog events.
pub struct IccProfileCatalog {
    store: IccProfileStore,
    state: Mutex<CatalogState>,
}

impl IccProfileCatalog {
    /// Opens the content-addressed profile store and an empty in-memory catalog.
    ///
    /// Existing profile files are intentionally not guessed into descriptors:
    /// catalog metadata must be restored from a trusted manifest in a later
    /// persistence layer rather than inferred from filenames.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, ProfileStoreError> {
        Ok(Self {
            store: IccProfileStore::open(root)?,
            state: Mutex::new(CatalogState::default()),
        })
    }

    /// Installs a profile supplied by the trusted AureaOS system image.
    ///
    /// This entry point is for system provisioning only, never for user imports.
    /// System profiles are marked read-only and cannot be removed through the
    /// regular profile manager interface.
    pub fn install_system_profile(
        &self,
        profile: IccProfileImport,
    ) -> Result<IccProfileDescriptor, IccProfileManagerError> {
        self.install(profile, IccProfileOrigin::System)
    }

    /// Registers a display discovered by the active output backend.
    pub fn register_display(&self, display_id: DisplayId) -> Result<(), IccProfileManagerError> {
        let mut state = self.lock_state()?;
        state.connected_displays.insert(display_id);
        Ok(())
    }

    /// Unregisters a disconnected display but preserves its profile assignment.
    pub fn unregister_display(&self, display_id: DisplayId) -> Result<(), IccProfileManagerError> {
        let mut state = self.lock_state()?;
        state.connected_displays.remove(&display_id);
        Ok(())
    }

    fn lock_state(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, CatalogState>, IccProfileManagerError> {
        self.state
            .lock()
            .map_err(|_| IccProfileManagerError::StorageFailure)
    }

    fn publish(state: &mut CatalogState) {
        let snapshot = snapshot_from(state);
        state
            .subscribers
            .retain(|subscriber| subscriber.send(snapshot.clone()).is_ok());
    }

    fn install(
        &self,
        profile: IccProfileImport,
        origin: IccProfileOrigin,
    ) -> Result<IccProfileDescriptor, IccProfileManagerError> {
        if profile.bytes.is_empty() {
            return Err(IccProfileManagerError::EmptyProfile);
        }
        if profile.bytes.len() > MAX_ICC_PROFILE_BYTES {
            return Err(IccProfileManagerError::ProfileTooLarge {
                maximum_bytes: MAX_ICC_PROFILE_BYTES,
            });
        }

        let stored = self.store.store(&profile.bytes).map_err(map_store_error)?;
        let id = IccProfileId::new(format!("sha256:{}", stored.fingerprint.to_hex()))
            .ok_or(IccProfileManagerError::InvalidProfile)?;
        let key = id.as_str().to_owned();
        let descriptor = descriptor_for(id, &stored.summary, profile.format, origin);

        let mut state = self.lock_state()?;
        if let Some(existing) = state.profiles.get(&key) {
            return Ok(existing.descriptor.clone());
        }
        state.profiles.insert(
            key,
            CatalogEntry {
                descriptor: descriptor.clone(),
                fingerprint: stored.fingerprint,
            },
        );
        Self::publish(&mut state);
        Ok(descriptor)
    }
}

impl IccProfileProvider for IccProfileCatalog {
    fn snapshot(&self) -> IccProfileCatalogSnapshot {
        self.state
            .lock()
            .map(|state| snapshot_from(&state))
            .unwrap_or_default()
    }

    fn subscribe(&self) -> Receiver<IccProfileCatalogSnapshot> {
        let (sender, receiver) = mpsc::channel();
        if let Ok(mut state) = self.state.lock() {
            let _ = sender.send(snapshot_from(&state));
            state.subscribers.push(sender);
        }
        receiver
    }
}

impl IccProfileManager for IccProfileCatalog {
    fn import_profile(
        &self,
        profile: IccProfileImport,
    ) -> Result<IccProfileDescriptor, IccProfileManagerError> {
        self.install(profile, IccProfileOrigin::Imported)
    }

    fn install_downloaded_profile(
        &self,
        profile: IccProfileImport,
    ) -> Result<IccProfileDescriptor, IccProfileManagerError> {
        self.install(profile, IccProfileOrigin::Downloaded)
    }

    fn remove_profile(&self, profile_id: &IccProfileId) -> Result<(), IccProfileManagerError> {
        let mut state = self.lock_state()?;
        let Some(entry) = state.profiles.get(profile_id.as_str()) else {
            return Err(IccProfileManagerError::ProfileNotFound);
        };
        if entry.descriptor.read_only || entry.descriptor.origin == IccProfileOrigin::System {
            return Err(IccProfileManagerError::SystemProfileProtected);
        }
        if state
            .assignments
            .values()
            .any(|assigned| assigned == profile_id)
        {
            return Err(IccProfileManagerError::AssignmentRejected);
        }
        let fingerprint = entry.fingerprint.clone();
        self.store.remove(&fingerprint).map_err(map_store_error)?;
        state.profiles.remove(profile_id.as_str());
        Self::publish(&mut state);
        Ok(())
    }

    fn create_editable_copy(
        &self,
        _source_profile_id: &IccProfileId,
        _new_name: String,
    ) -> Result<IccProfileDescriptor, IccProfileManagerError> {
        // ICC editing must generate new profile bytes. Copying the original
        // bytes would retain the same content identity and falsely imply edits.
        Err(IccProfileManagerError::UnsupportedOperation)
    }

    fn export_profile(&self, profile_id: &IccProfileId) -> Result<Vec<u8>, IccProfileManagerError> {
        let fingerprint = {
            let state = self.lock_state()?;
            state
                .profiles
                .get(profile_id.as_str())
                .map(|entry| entry.fingerprint.clone())
                .ok_or(IccProfileManagerError::ProfileNotFound)?
        };
        self.store.read_bytes(&fingerprint).map_err(map_store_error)
    }

    fn assign_profile(
        &self,
        assignment: DisplayIccProfileAssignment,
    ) -> Result<(), IccProfileManagerError> {
        let mut state = self.lock_state()?;
        if !state.connected_displays.contains(&assignment.display_id) {
            return Err(IccProfileManagerError::DisplayNotFound);
        }
        if let Some(profile_id) = &assignment.profile_id {
            let entry = state
                .profiles
                .get(profile_id.as_str())
                .ok_or(IccProfileManagerError::ProfileNotFound)?;
            if entry.descriptor.profile_class != IccProfileClass::Display {
                return Err(IccProfileManagerError::UnsupportedProfileClass);
            }
            state
                .assignments
                .insert(assignment.display_id, profile_id.clone());
        } else {
            state.assignments.remove(&assignment.display_id);
        }
        Self::publish(&mut state);
        Ok(())
    }

    fn clear_assignment(&self, display_id: DisplayId) -> Result<(), IccProfileManagerError> {
        let mut state = self.lock_state()?;
        state.assignments.remove(&display_id);
        Self::publish(&mut state);
        Ok(())
    }
}

fn snapshot_from(state: &CatalogState) -> IccProfileCatalogSnapshot {
    let mut profiles = state
        .profiles
        .values()
        .map(|entry| entry.descriptor.clone())
        .collect::<Vec<_>>();
    profiles.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
    let assignments = state
        .assignments
        .iter()
        .map(|(display_id, profile_id)| DisplayIccProfileAssignment {
            display_id: *display_id,
            profile_id: Some(profile_id.clone()),
        })
        .collect();
    IccProfileCatalogSnapshot {
        profiles,
        assignments,
    }
}

fn descriptor_for(
    id: IccProfileId,
    summary: &crate::IccProfileSummary,
    format: IccProfileFormat,
    origin: IccProfileOrigin,
) -> IccProfileDescriptor {
    let profile_class = match &summary.device_class {
        b"scnr" => IccProfileClass::Input,
        b"mntr" => IccProfileClass::Display,
        b"prtr" => IccProfileClass::Output,
        b"link" => IccProfileClass::DeviceLink,
        b"spac" => IccProfileClass::ColorSpace,
        b"abst" => IccProfileClass::Abstract,
        b"nmcl" => IccProfileClass::NamedColor,
        _ => IccProfileClass::Unknown,
    };
    let short_id = id.as_str()[7..19.min(id.as_str().len())].to_owned();
    IccProfileDescriptor {
        id,
        name: format!("ICC profile {short_id}"),
        description: None,
        manufacturer: signature_text(&summary.manufacturer),
        model: signature_text(&summary.model),
        copyright: None,
        profile_class,
        origin,
        format,
        parent_profile_id: None,
        read_only: origin == IccProfileOrigin::System,
    }
}

fn signature_text(value: &[u8; 4]) -> Option<String> {
    if value
        .iter()
        .all(|byte| *byte == 0 || byte.is_ascii_graphic() || *byte == b' ')
    {
        let text = String::from_utf8_lossy(value).trim().to_owned();
        if text.is_empty() { None } else { Some(text) }
    } else {
        None
    }
}

fn map_store_error(error: ProfileStoreError) -> IccProfileManagerError {
    match error {
        ProfileStoreError::InvalidProfile => IccProfileManagerError::InvalidProfile,
        ProfileStoreError::ProfileTooLarge => IccProfileManagerError::ProfileTooLarge {
            maximum_bytes: MAX_ICC_PROFILE_BYTES,
        },
        ProfileStoreError::ProfileNotFound => IccProfileManagerError::ProfileNotFound,
        ProfileStoreError::UnsafeStorageEntry | ProfileStoreError::Io(_) => {
            IccProfileManagerError::StorageFailure
        }
    }
}

#[cfg(test)]
#[path = "profile_catalog/tests.rs"]
mod tests;
