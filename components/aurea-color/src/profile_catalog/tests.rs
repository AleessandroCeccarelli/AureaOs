// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Unit tests for ICC profile catalog behavior.

use super::*;

    use super::*;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT: AtomicU64 = AtomicU64::new(0);

    fn test_catalog() -> (IccProfileCatalog, std::path::PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "aurea-icc-catalog-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        (IccProfileCatalog::open(&root).unwrap(), root)
    }

    fn display_profile() -> IccProfileImport {
        let mut bytes = vec![0_u8; 132];
        bytes[0..4].copy_from_slice(&132_u32.to_be_bytes());
        bytes[8..12].copy_from_slice(b"\x04\x30\0\0");
        bytes[12..16].copy_from_slice(b"mntr");
        bytes[16..20].copy_from_slice(b"RGB ");
        bytes[20..24].copy_from_slice(b"XYZ ");
        bytes[36..40].copy_from_slice(b"acsp");
        IccProfileImport {
            format: IccProfileFormat::Icc,
            bytes,
        }
    }

    #[test]
    fn duplicate_import_is_idempotent_and_notifies_subscribers() {
        let (catalog, root) = test_catalog();
        let receiver = catalog.subscribe();
        let _initial = receiver.recv().unwrap();
        let first = catalog.import_profile(display_profile()).unwrap();
        let event = receiver.recv().unwrap();
        let second = catalog.import_profile(display_profile()).unwrap();
        assert_eq!(first.id, second.id);
        assert_eq!(event.profiles.len(), 1);
        assert_eq!(catalog.snapshot().profiles.len(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn assignments_are_independent_per_connected_display() {
        let (catalog, root) = test_catalog();
        let profile = catalog.import_profile(display_profile()).unwrap();
        let first = DisplayId(1);
        let second = DisplayId(2);
        catalog.register_display(first).unwrap();
        catalog.register_display(second).unwrap();
        for display_id in [first, second] {
            catalog
                .assign_profile(DisplayIccProfileAssignment {
                    display_id,
                    profile_id: Some(profile.id.clone()),
                })
                .unwrap();
        }
        let snapshot = catalog.snapshot();
        assert_eq!(snapshot.assignments.len(), 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_unknown_profile_and_disconnected_output() {
        let (catalog, root) = test_catalog();
        let missing = IccProfileId::new("sha256:missing").unwrap();
        assert_eq!(
            catalog.assign_profile(DisplayIccProfileAssignment {
                display_id: DisplayId(9),
                profile_id: Some(missing),
            }),
            Err(IccProfileManagerError::DisplayNotFound)
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn imported_profile_can_be_exported_and_removed() {
        let (catalog, root) = test_catalog();
        let profile = catalog.import_profile(display_profile()).unwrap();
        assert_eq!(
            catalog.export_profile(&profile.id).unwrap(),
            display_profile().bytes
        );
        catalog.remove_profile(&profile.id).unwrap();
        assert_eq!(
            catalog.export_profile(&profile.id),
            Err(IccProfileManagerError::ProfileNotFound)
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn trusted_system_profiles_are_read_only_and_cannot_be_removed() {
        let (catalog, root) = test_catalog();
        let profile = catalog.install_system_profile(display_profile()).unwrap();
        assert_eq!(profile.origin, IccProfileOrigin::System);
        assert!(profile.read_only);
        assert_eq!(
            catalog.remove_profile(&profile.id),
            Err(IccProfileManagerError::SystemProfileProtected)
        );
        assert_eq!(catalog.snapshot().profiles.len(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_malformed_profile_without_catalog_entry() {
        let (catalog, root) = test_catalog();
        let invalid = IccProfileImport {
            format: IccProfileFormat::Icc,
            bytes: b"bad".to_vec(),
        };
        assert_eq!(
            catalog.import_profile(invalid),
            Err(IccProfileManagerError::InvalidProfile)
        );
        assert!(catalog.snapshot().profiles.is_empty());
        fs::remove_dir_all(root).unwrap();
    }
