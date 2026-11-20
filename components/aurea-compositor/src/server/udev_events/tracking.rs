// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Small identity and retry collections used by the Udev lifecycle.

use std::path::PathBuf;

pub(in crate::server) fn take_notifier_token<K: Eq + std::hash::Hash, V>(
    map: &mut std::collections::HashMap<K, V>,
    key: &K,
) -> Option<V> {
    map.remove(key)
}

pub(in crate::server) fn track_device<N: Copy + Eq>(
    devices: &mut Vec<(N, PathBuf)>,
    node: N,
    path: PathBuf,
) -> bool {
    if devices.iter().any(|(known, _)| *known == node) {
        return false;
    }

    devices.push((node, path));

    true
}

pub(in crate::server) fn forget_device<N: Copy + Eq>(
    devices: &mut Vec<(N, PathBuf)>,
    node: N,
) -> Option<PathBuf> {
    devices
        .iter()
        .position(|(known, _)| *known == node)
        .map(|index| devices.remove(index).1)
}

pub(in crate::server) fn queue_changed_device<N: Copy + Eq>(changed: &mut Vec<N>, node: N) {
    if !changed.contains(&node) {
        changed.push(node);
    }
}

pub(in crate::server) fn forget_changed_device<N: Copy + Eq>(changed: &mut Vec<N>, node: N) {
    changed.retain(|known| *known != node);
}

pub(in crate::server) fn retry_pending_changes<N, E, F>(
    pending: Vec<N>,
    managed: &std::collections::HashSet<N>,
    mut refresh: F,
) -> (Vec<N>, Vec<(N, E)>)
where
    N: Copy + Eq + std::hash::Hash,
    F: FnMut(N) -> Result<(), E>,
{
    let mut retained = Vec::new();
    let mut failures = Vec::new();

    for node in pending {
        if !managed.contains(&node) {
            retained.push(node);
            continue;
        }

        match refresh(node) {
            Ok(()) => {}
            Err(error) => {
                retained.push(node);
                failures.push((node, error));
            }
        }
    }

    (retained, failures)
}
