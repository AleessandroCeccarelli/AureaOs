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

/// Validates a Wayland socket basename before it reaches the runtime directory.
///
/// Linux `sockaddr_un.sun_path` is 108 bytes including the trailing NUL, so
/// the socket name itself is limited to 107 bytes. Restricting the alphabet
/// also prevents path separators and control characters from escaping the
/// intended runtime-directory namespace.
fn validate_socket_name(socket_name: &str) -> io::Result<()> {
    const MAX_SOCKET_NAME_BYTES: usize = 107;

    let valid = !socket_name.is_empty()
        && socket_name.len() <= MAX_SOCKET_NAME_BYTES
        && socket_name != "."
        && socket_name != ".."
        && socket_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));

    if valid {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Wayland socket name must be a valid basename of at most 107 ASCII bytes",
        ))
    }
}

/// Creates the listening socket and registers incoming Wayland clients.
pub(super) fn register_wayland_socket(
    loop_handle: &LoopHandle<'_, WaylandState>,
    socket_name: &str,
) -> io::Result<()> {
    validate_socket_name(socket_name)?;

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

#[cfg(test)]
mod tests {
    use super::validate_socket_name;
    use std::io::ErrorKind;

    #[test]
    fn accepts_standard_wayland_socket_names() {
        assert!(validate_socket_name("wayland-0").is_ok());
        assert!(validate_socket_name("aurea_session.1").is_ok());
    }

    #[test]
    fn rejects_empty_or_path_like_socket_names() {
        for name in [
            "",
            ".",
            "..",
            "../wayland-0",
            "nested/socket",
            "/tmp/socket",
        ] {
            assert_eq!(
                validate_socket_name(name).unwrap_err().kind(),
                ErrorKind::InvalidInput,
                "accepted invalid socket name: {name:?}"
            );
        }
    }

    #[test]
    fn rejects_control_characters_and_oversized_names() {
        assert!(validate_socket_name("wayland-0\n").is_err());
        assert!(validate_socket_name(&"a".repeat(108)).is_err());
        assert!(validate_socket_name(&"a".repeat(107)).is_ok());
    }
}
