// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Bounds the number of protocol objects owned by each Wayland client.

use smithay::reexports::wayland_server::{DisplayHandle, backend::DisconnectReason};

const MAX_OBJECTS_PER_CLIENT: usize = 2048;
const MAX_OBJECTS_GLOBAL: usize = 16_384;

/// Disconnects clients that exceed per-client or compositor-wide object budgets.
///
/// This runs after each Wayland dispatch, so it covers objects that have not
/// yet reached a surface commit (which Smithay 0.7 does not expose as a hook).
pub(super) fn enforce_wayland_object_limits(display: &DisplayHandle) {
    let backend = display.backend_handle();
    let mut client_ids = Vec::new();
    backend.with_all_clients(|client_id| client_ids.push(client_id));

    let mut counts = Vec::with_capacity(client_ids.len());
    for client_id in client_ids {
        let mut object_count = 0usize;
        if backend
            .with_all_objects_for(client_id.clone(), |_| {
                object_count = object_count.saturating_add(1);
            })
            .is_ok()
        {
            counts.push((client_id, object_count));
        }
    }

    for client_id in select_clients_to_disconnect(counts) {
        backend.kill_client(client_id, DisconnectReason::ConnectionClosed);
    }
}

fn select_clients_to_disconnect<Client>(mut counts: Vec<(Client, usize)>) -> Vec<Client> {
    let mut total = counts
        .iter()
        .fold(0usize, |sum, (_, count)| sum.saturating_add(*count));
    counts.sort_by(|left, right| right.1.cmp(&left.1));

    let mut disconnected = Vec::new();
    for (client_id, count) in counts {
        if count > MAX_OBJECTS_PER_CLIENT || total > MAX_OBJECTS_GLOBAL {
            total = total.saturating_sub(count);
            disconnected.push(client_id);
        }
    }

    disconnected
}

#[cfg(test)]
mod tests {
    use super::{MAX_OBJECTS_GLOBAL, MAX_OBJECTS_PER_CLIENT, select_clients_to_disconnect};

    #[test]
    fn object_budget_accepts_clients_below_both_limits() {
        let clients = vec![(1_u8, 100), (2_u8, 200)];
        assert!(select_clients_to_disconnect(clients).is_empty());
    }

    #[test]
    fn object_budget_disconnects_a_client_over_its_individual_limit() {
        let clients = vec![(1_u8, MAX_OBJECTS_PER_CLIENT + 1), (2_u8, 100)];
        assert_eq!(select_clients_to_disconnect(clients), vec![1]);
    }

    #[test]
    fn object_budget_reduces_global_exhaustion() {
        let clients = (0_u8..9)
            .map(|client| (client, MAX_OBJECTS_GLOBAL / 8))
            .collect::<Vec<_>>();
        let disconnected = select_clients_to_disconnect(clients);

        assert_eq!(disconnected.len(), 1);
    }

    #[test]
    fn exact_individual_and_global_limits_are_accepted() {
        let clients = (0_u8..8)
            .map(|client| (client, MAX_OBJECTS_PER_CLIENT))
            .collect::<Vec<_>>();

        assert!(select_clients_to_disconnect(clients).is_empty());
    }
}
