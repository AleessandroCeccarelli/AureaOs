
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Wayland server lifecycle and client dispatch.

use std::sync::Arc;

use smithay::{
    delegate_compositor,
    reexports::wayland_server::{
        backend::{ClientData, ClientId, DisconnectReason},
        protocol::wl_surface::WlSurface,
        Client, Display, ListeningSocket,
    },
    wayland::compositor::{
        CompositorClientState, CompositorHandler, CompositorState,
    },
};

use crate::AureaCompositor;

/// Per-client protocol state.
#[derive(Default)]
pub struct AureaClientState {
    compositor_state: CompositorClientState,
}

impl ClientData for AureaClientState {
    fn initialized(&self, _client_id: ClientId) {}

    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

impl CompositorHandler for AureaCompositor {
    fn compositor_state(&mut self) -> &mut CompositorState {
        self.compositor_state()
    }

    fn client_compositor_state<'a>(
        &self,
        client: &'a Client,
    ) -> &'a CompositorClientState {
        &client
            .get_data::<AureaClientState>()
            .expect("AureaClientState must be attached to every client")
            .compositor_state
    }

    fn commit(&mut self, _surface: &WlSurface) {}
}

delegate_compositor!(AureaCompositor);

/// Owns the Wayland display, listening socket, and connected clients.
pub struct WaylandServer {
    display: Display<AureaCompositor>,
    compositor: AureaCompositor,
    listener: ListeningSocket,
    clients: Vec<Client>,
}

impl WaylandServer {
    /// Creates a server bound to the requested Wayland socket name.
    pub fn new(socket_name: &str) -> std::io::Result<Self> {
        let mut display = Display::<AureaCompositor>::new()?;
        let display_handle = display.handle();
        let compositor = AureaCompositor::new(&display_handle);
        let listener = ListeningSocket::bind(socket_name)?;

        Ok(Self {
            display,
            compositor,
            listener,
            clients: Vec::new(),
        })
    }

    /// Accepts pending clients and dispatches their protocol requests.
    pub fn dispatch(&mut self) -> std::io::Result<()> {
        while let Some(stream) = self.listener.accept()? {
            let client = self
                .display
                .handle()
                .insert_client(stream, Arc::new(AureaClientState::default()))?;
            self.clients.push(client);
        }

        self.display.dispatch_clients(&mut self.compositor)?;
        self.display.flush_clients()
    }
}
