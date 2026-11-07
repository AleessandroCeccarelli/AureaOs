// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS test module for Udev DRM lifecycle helpers.

use super::*;
use std::collections::HashMap;

#[test]
fn duplicate_added_identity_does_not_duplicate_inventory() {
    let path = PathBuf::from("/dev/dri/card-test");
    let mut devices = Vec::new();

    assert!(track_device(&mut devices, 11_u32, path.clone()));
    assert!(!track_device(
        &mut devices,
        11_u32,
        PathBuf::from("/dev/dri/card-other")
    ));
    assert_eq!(devices, vec![(11, path)]);
}

#[test]
fn shared_path_is_retained_until_its_last_identity_is_removed() {
    let path = PathBuf::from("/dev/dri/card-test");
    let mut devices = vec![(11_u32, path.clone()), (12_u32, path.clone())];

    assert_eq!(forget_device(&mut devices, 11), Some(path.clone()));
    assert!(devices.iter().any(|(_, existing)| existing == &path));
    assert_eq!(forget_device(&mut devices, 12), Some(path.clone()));
    assert!(!devices.iter().any(|(_, existing)| existing == &path));
}

#[test]
fn shared_gpu_inventory_entry_is_removed_only_for_the_final_identity() {
    let path = PathBuf::from("/dev/dri/card-test");
    let mut devices = vec![(11_u32, path.clone()), (12_u32, path.clone())];
    let mut gpus = crate::gpu::GpuManager::new();
    gpus.register_device(path.clone()).unwrap();

    if let Some(removed) = forget_device(&mut devices, 11) {
        if !devices.iter().any(|(_, existing)| existing == &removed) {
            gpus.remove_device(&removed);
        }
    }
    assert!(gpus.get_device(&path).is_some());

    if let Some(removed) = forget_device(&mut devices, 12) {
        if !devices.iter().any(|(_, existing)| existing == &removed) {
            gpus.remove_device(&removed);
        }
    }
    assert!(gpus.get_device(&path).is_none());
}

#[test]
fn changed_events_are_deduplicated_and_removed_entries_are_cleared() {
    let mut changed = Vec::new();
    queue_changed_device(&mut changed, 11_u32);
    queue_changed_device(&mut changed, 11_u32);
    queue_changed_device(&mut changed, 12_u32);

    assert_eq!(changed, vec![11, 12]);
    forget_changed_device(&mut changed, 11);
    assert_eq!(changed, vec![12]);
}

#[test]
fn output_identity_filter_only_matches_its_own_device() {
    let first = (7_u128 << 32) | 3;
    let second = (8_u128 << 32) | 3;

    assert!(output_belongs_to_device(first, 7));
    assert!(!output_belongs_to_device(second, 7));
}

#[test]
fn repeated_removal_and_notifier_bookkeeping_are_safe() {
    let mut devices = vec![(11_u32, PathBuf::from("/dev/dri/card-test"))];
    let mut notifiers = HashMap::from([(11_u32, 99_u32)]);

    assert!(forget_device(&mut devices, 11).is_some());
    assert!(forget_device(&mut devices, 11).is_none());
    assert_eq!(take_notifier_token(&mut notifiers, &11), Some(99));
    assert_eq!(take_notifier_token(&mut notifiers, &11), None);
    assert!(notifiers.is_empty());
}

#[test]
fn deferred_refresh_keeps_failures_and_unmanaged_devices_but_removes_successes() {
    let pending = vec![11_u32, 12, 13];
    let managed = [11_u32, 12].into_iter().collect();
    let (remaining, failures) = retry_pending_changes(pending, &managed, |node| {
        if node == 11 {
            Err("transient refresh error")
        } else {
            Ok(())
        }
    });

    assert_eq!(remaining, vec![11, 13]);
    assert_eq!(failures, vec![(11, "transient refresh error")]);
}
