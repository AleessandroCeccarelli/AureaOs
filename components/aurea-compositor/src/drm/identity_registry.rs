// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Runtime index of physical identity evidence for connected DRM outputs.

use crate::output::OutputId;
use aurea_desktop_api::display::DisplayIdentityEvidence;
use std::collections::HashMap;

/// Associates transient output IDs with current hardware evidence.
/// OutputId is deliberately not a persistent monitor identifier.
#[derive(Debug, Default)]
pub struct DrmDisplayIdentityRegistry {
    entries: HashMap<OutputId, DisplayIdentityEvidence>,
}

impl DrmDisplayIdentityRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces evidence for one DRM device after successful reconciliation.
    pub fn replace_device(
        &mut self,
        device_id: u64,
        evidence: impl IntoIterator<Item = (OutputId, DisplayIdentityEvidence)>,
    ) {
        self.entries.retain(|id, _| (*id >> 32) as u64 != device_id);
        self.entries.extend(
            evidence
                .into_iter()
                .filter(|(id, _)| (*id >> 32) as u64 == device_id),
        );
    }

    pub fn remove(&mut self, output_id: OutputId) {
        self.entries.remove(&output_id);
    }
    pub fn get(&self, output_id: OutputId) -> Option<&DisplayIdentityEvidence> {
        self.entries.get(&output_id)
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::DrmDisplayIdentityRegistry;
    use aurea_desktop_api::display::DisplayIdentityEvidence;

    fn evidence(serial: &str) -> DisplayIdentityEvidence {
        DisplayIdentityEvidence::new("pci/DP-1", *b"DEL", 1, Some(serial.to_owned())).unwrap()
    }

    #[test]
    fn replaces_only_entries_owned_by_the_device() {
        let mut registry = DrmDisplayIdentityRegistry::new();
        registry.replace_device(1, [(1_u128 << 32 | 2, evidence("A"))]);
        registry.replace_device(2, [(2_u128 << 32 | 3, evidence("B"))]);
        registry.replace_device(1, [(1_u128 << 32 | 4, evidence("C"))]);
        assert!(registry.get(1_u128 << 32 | 2).is_none());
        assert!(registry.get(1_u128 << 32 | 4).is_some());
        assert!(registry.get(2_u128 << 32 | 3).is_some());
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn removes_disconnected_runtime_output() {
        let mut registry = DrmDisplayIdentityRegistry::new();
        let id = 1_u128 << 32 | 2;
        registry.replace_device(1, [(id, evidence("A"))]);
        registry.remove(id);
        assert!(registry.is_empty());
    }
}
