// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Wayland listening socket and client registration.

use std::{io, sync::Arc};

use smithay::reexports::calloop::LoopHandle;
use smithay::wayland::socket::ListeningSocketSource;

use super::{client::AureaClientState, state::WaylandState};

/// Maximum number of simultaneous local Wayland clients accepted by the server.
///
/// Disconnected clients are removed by the display event handler, so this
/// limit applies to live clients rather than the lifetime number of connections.
const MAX_WAYLAND_CLIENTS: usize = 64;

/// Creates the listening socket and registers incoming Wayland clients.
pub(super) fn register_wayland_socket(
    loop_handle: &LoopHandle<'_, WaylandState>,
    socket_name: &str,
) -> io::Result<()> {
    let listening_socket =
        ListeningSocketSource::with_name(socket_name).map_err(io::Error::other)?;

    loop_handle
        .insert_source(listening_socket, |stream, _, state| {
            // Drop excess connections before allocating client state or
            // registering another Wayland client.
            if state.clients.len() >= MAX_WAYLAND_CLIENTS {
                drop(stream);
                return;
            }

            let client_state = AureaClientState::new(Arc::clone(&state.disconnected_clients));

            match state
                .display
                .handle()
                .insert_client(stream, Arc::new(client_state))
            {
                Ok(client) => state.clients.push(client),
                Err(error) => {
                    state.error = Some(io::Error::other(error));
                }
            }
        })
        .map_err(io::Error::other)?;

    Ok(())
}
