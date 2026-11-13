// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Bounds Wayland surfaces after their first client commit.

use std::collections::HashMap;

use smithay::reexports::wayland_server::backend::{ClientId, ObjectId};

/// Maximum number of committed wl_surface objects owned by one client.
const MAX_SURFACES_PER_CLIENT: usize = 256;

/// Maximum number of committed wl_surface objects across the compositor.
const MAX_SURFACES_GLOBAL: usize = 4096;

/// Tracks accepted surfaces so disconnected or destroyed clients release quota.
#[derive(Default)]
pub(super) struct WaylandResourceLimits {
    surface_owners: HashMap<ObjectId, ClientId>,
    client_surface_counts: HashMap<ClientId, usize>,
}

impl WaylandResourceLimits {
    pub(super) fn contains_surface(&self, surface: &ObjectId) -> bool {
        self.surface_owners.contains_key(surface)
    }

    pub(super) fn register_surface(&mut self, surface: ObjectId, client: ClientId) -> bool {
        if self.surface_owners.contains_key(&surface) {
            return false;
        }

        let client_count = self.client_surface_counts.get(&client).copied().unwrap_or(0);
        if !within_surface_limits(self.surface_owners.len(), client_count) {
            return false;
        }

        self.surface_owners.insert(surface, client.clone());
        *self.client_surface_counts.entry(client).or_default() += 1;
        true
    }

    pub(super) fn unregister_surface(&mut self, surface: &ObjectId) {
        let Some(client) = self.surface_owners.remove(surface) else {
            return;
        };

        let remove_client = if let Some(count) = self.client_surface_counts.get_mut(&client) {
            *count = count.saturating_sub(1);
            *count == 0
        } else {
            false
        };

        if remove_client {
            self.client_surface_counts.remove(&client);
        }
    }
}

fn within_surface_limits(global_count: usize, client_count: usize) -> bool {
    global_count < MAX_SURFACES_GLOBAL && client_count < MAX_SURFACES_PER_CLIENT
}

#[cfg(test)]
mod tests {
    use super::{MAX_SURFACES_GLOBAL, MAX_SURFACES_PER_CLIENT, within_surface_limits};

    #[test]
    fn surface_limits_accept_values_below_both_caps() {
        assert!(within_surface_limits(
            MAX_SURFACES_GLOBAL - 1,
            MAX_SURFACES_PER_CLIENT - 1
        ));
    }

    #[test]
    fn surface_limits_reject_global_exhaustion() {
        assert!(!within_surface_limits(
            MAX_SURFACES_GLOBAL,
            MAX_SURFACES_PER_CLIENT - 1
        ));
    }

    #[test]
    fn surface_limits_reject_per_client_exhaustion() {
        assert!(!within_surface_limits(
            MAX_SURFACES_GLOBAL - 1,
            MAX_SURFACES_PER_CLIENT
        ));
    }
}
