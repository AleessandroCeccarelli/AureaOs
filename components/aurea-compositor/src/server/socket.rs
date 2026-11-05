
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Wayland listening socket and client registration.

use std::io;

use smithay::reexports::calloop::LoopHandle;
use smithay::wayland::socket::ListeningSocketSource;

use super::{client::AureaClientState, state::WaylandState};

/// Creates the listening socket and registers incoming Wayland clients.
pub(super) fn register_wayland_socket(
    loop_handle: &LoopHandle<'_, WaylandState>,
    socket_name: &str,
) -> io::Result<()> {
    let listening_socket =
        ListeningSocketSource::with_name(socket_name).map_err(io::Error::other)?;

    loop_handle
        .insert_source(listening_socket, |stream, _, state| {
            match state
                .display
                .handle()
                .insert_client(stream, std::sync::Arc::new(AureaClientState::default()))
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
