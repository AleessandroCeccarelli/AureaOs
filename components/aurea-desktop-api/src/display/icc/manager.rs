// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Mutating operations for the ICC profile catalog.

use super::{
    DisplayId, DisplayIccProfileAssignment, IccProfileDescriptor, IccProfileFormat,
    IccProfileId,
};

/// Upper bound for an imported ICC profile, preventing unbounded allocations.
pub const MAX_ICC_PROFILE_BYTES: usize = 16 * 1024 * 1024;

/// Request to import a profile supplied by the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IccProfileImport {
    pub format: IccProfileFormat,
    pub bytes: Vec<u8>,
}

/// Failures reported by the profile manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IccProfileManagerError {
    EmptyProfile,
    ProfileTooLarge { maximum_bytes: usize },
    InvalidProfile,
    UnsupportedProfileClass,
    ProfileNotFound,
    DisplayNotFound,
    SystemProfileProtected,
    AssignmentRejected,
    StorageFailure,
    ExportFailure,
}

/// Service that owns ICC profile storage and per-display assignments.
///
/// Implementations must parse and validate profile contents before persisting
/// them. Imports and downloaded profiles are stored as immutable source
/// artifacts. Editing must create a separate user-owned copy.
pub trait IccProfileManager: Send + Sync {
    /// Validates and installs a profile imported from local storage.
    fn import_profile(
        &self,
        profile: IccProfileImport,
    ) -> Result<IccProfileDescriptor, IccProfileManagerError>;

    /// Installs bytes already obtained by a trusted download workflow.
    ///
    /// Network retrieval, origin verification and consent are handled outside
    /// this API. The manager still validates the complete ICC payload.
    fn install_downloaded_profile(
        &self,
        profile: IccProfileImport,
    ) -> Result<IccProfileDescriptor, IccProfileManagerError>;

    /// Removes a user-owned profile. System profiles cannot be removed.
    fn remove_profile(&self, profile_id: &IccProfileId) -> Result<(), IccProfileManagerError>;

    /// Creates an independent editable copy while preserving the source.
    fn create_editable_copy(
        &self,
        source_profile_id: &IccProfileId,
        new_name: String,
    ) -> Result<IccProfileDescriptor, IccProfileManagerError>;

    /// Exports the exact stored profile bytes without modifying the catalog.
    fn export_profile(
        &self,
        profile_id: &IccProfileId,
    ) -> Result<Vec<u8>, IccProfileManagerError>;

    /// Assigns a validated display-class profile to a connected display.
    fn assign_profile(
        &self,
        assignment: DisplayIccProfileAssignment,
    ) -> Result<(), IccProfileManagerError>;

    /// Clears the ICC assignment for one display.
    fn clear_assignment(&self, display_id: DisplayId) -> Result<(), IccProfileManagerError>;
}

#[cfg(test)]
mod tests {
    use super::MAX_ICC_PROFILE_BYTES;
    use crate::display::IccProfileOrigin;

    #[test]
    fn import_size_limit_is_suitable_for_profile_payloads() {
        assert!(MAX_ICC_PROFILE_BYTES >= 4 * 1024 * 1024);
        assert!(MAX_ICC_PROFILE_BYTES <= 64 * 1024 * 1024);
    }

    #[test]
    fn imported_and_downloaded_profiles_have_distinct_origins() {
        assert_ne!(
            IccProfileOrigin::Imported,
            IccProfileOrigin::Downloaded
        );
    }
}
