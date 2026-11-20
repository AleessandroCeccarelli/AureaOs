// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Technology-independent contracts for ICC display profiles.
//!
//! Profile bytes, parsing, persistence, downloads and LittleCMS transforms are
//! owned by dedicated services. This module only describes profile identity,
//! metadata and per-display assignment.

use std::sync::mpsc::Receiver;

use super::DisplayId;

pub mod manager;
pub use manager::{IccProfileImport, IccProfileManager, IccProfileManagerError, MAX_ICC_PROFILE_BYTES};

/// Stable identifier assigned by the profile service to an ICC profile.
///
/// The identifier is opaque to API consumers. Implementations should derive it
/// from the profile content so importing the same profile twice is idempotent.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IccProfileId(String);

impl IccProfileId {
    /// Creates an identifier from a non-empty service-generated value.
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return None;
        }
        Some(Self(value))
    }

    /// Returns the opaque identifier value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// ICC profile class as declared by the profile header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IccProfileClass {
    Input,
    Display,
    Output,
    DeviceLink,
    ColorSpace,
    Abstract,
    NamedColor,
    Unknown,
}

/// How a profile entered the local AureaOS profile catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IccProfileOrigin {
    /// Profile shipped and maintained by AureaOS.
    System,
    /// Profile imported by the user from local storage.
    Imported,
    /// Profile downloaded by the user from an external source.
    Downloaded,
    /// Profile created or exported by the user with AureaOS tools.
    Generated,
}

/// File extension accepted by the import/export interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IccProfileFormat {
    Icc,
    Icm,
}

/// Descriptive metadata for a validated profile in the local catalog.
///
/// The descriptor deliberately contains no filesystem path or profile bytes.
/// Those remain private to the profile storage service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IccProfileDescriptor {
    pub id: IccProfileId,
    pub name: String,
    pub description: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub copyright: Option<String>,
    pub profile_class: IccProfileClass,
    pub origin: IccProfileOrigin,
    pub format: IccProfileFormat,
    /// Original profile from which this user-generated profile was derived.
    /// The source remains untouched when a user creates an edited copy.
    pub parent_profile_id: Option<IccProfileId>,
    /// System profiles are protected from user deletion or in-place editing.
    pub read_only: bool,
}

/// ICC profile selected for one physical or virtual display output.
///
/// None means that the output uses the system's neutral/default color path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayIccProfileAssignment {
    pub display_id: DisplayId,
    pub profile_id: Option<IccProfileId>,
}

/// Consistent read-only view of the installed profile catalog and assignments.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IccProfileCatalogSnapshot {
    pub profiles: Vec<IccProfileDescriptor>,
    pub assignments: Vec<DisplayIccProfileAssignment>,
}

/// Read-only profile catalog contract shared with shell settings and services.
///
/// Mutating operations such as import, download, removal and assignment are
/// deliberately exposed by a separate manager service, not by this view.
pub trait IccProfileProvider: Send + Sync {
    /// Returns a consistent snapshot of profiles and display assignments.
    fn snapshot(&self) -> IccProfileCatalogSnapshot;
    /// Subscribes to catalog or assignment changes.
    fn subscribe(&self) -> Receiver<IccProfileCatalogSnapshot>;
}

#[cfg(test)]
mod tests {
    use super::{IccProfileId, IccProfileOrigin};

    #[test]
    fn profile_id_rejects_empty_values() {
        assert!(IccProfileId::new("").is_none());
        assert!(IccProfileId::new("   ").is_none());
    }

    #[test]
    fn profile_id_preserves_opaque_value() {
        let id = IccProfileId::new("sha256:abc123").unwrap();
        assert_eq!(id.as_str(), "sha256:abc123");
    }

    #[test]
    fn profile_origins_distinguish_user_downloads_from_imports() {
        assert_ne!(IccProfileOrigin::Imported, IccProfileOrigin::Downloaded);
    }
}
