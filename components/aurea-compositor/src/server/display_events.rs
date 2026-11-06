
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Wayland display file descriptor registration and client dispatch.

use std::{
    io,
    os::fd::OwnedFd,
};

use smithay::reexports::calloop::{
    generic::Generic,
    Interest,
    LoopHandle,
    Mode,
};

use super::state::WaylandState;

/// Removes clients whose disconnection has been reported.
fn remove_disconnected_clients(state: &mut WaylandState) {
    let disconnected_clients = state
        .disconnected_clients
        .lock()
        .map(|mut clients| std::mem::take(&mut *clients))
        .unwrap_or_default();

    if disconnected_clients.is_empty() {
        return;
    }

    state.clients.retain(|client| {
        !disconnected_clients.contains(&client.id().clone())
    });

    state
        .compositor
        .window_manager()
        .remove_dead_toplevels();
}

/// Registers the display fd and dispatches incoming client requests.
pub(super) fn register_display_events(
    loop_handle: &LoopHandle<'_, WaylandState>,
    display_fd: OwnedFd,
) -> io::Result<()> {
    let stop_handle = loop_handle.clone();

    loop_handle
        .insert_source(
            Generic::new(display_fd, Interest::READ, Mode::Level),
            move |_, _, state| {
                if let Err(error) =
                    state.display.dispatch_clients(&mut state.compositor)
                {
                    state.error = Some(io::Error::other(error));
                    stop_handle.stop();
                    return;
                }

                remove_disconnected_clients(state);

                if let Err(error) = state.display.flush_clients() {
                    state.error = Some(io::Error::other(error));
                    stop_handle.stop();
                }
            },
        )
        .map_err(io::Error::other)?;

    Ok(())
}
