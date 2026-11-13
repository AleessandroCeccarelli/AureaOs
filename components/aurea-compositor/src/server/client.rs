// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Per-client state and compositor protocol handling.

use std::sync::{Arc, Mutex};

use smithay::{
    delegate_compositor,
    reexports::wayland_server::{
        Client,
        backend::{ClientData, ClientId, DisconnectReason},
        protocol::wl_surface::WlSurface,
        Resource,
    },
    wayland::compositor::{
        BufferAssignment, CompositorClientState, CompositorHandler, CompositorState,
        SurfaceAttributes, add_destruction_hook, with_states,
    },
};

use crate::AureaCompositor;

/// State attached to each connected Wayland client.
pub(super) struct AureaClientState {
    compositor_state: CompositorClientState,
    disconnected_clients: Arc<Mutex<Vec<ClientId>>>,
}

impl AureaClientState {
    /// Creates client state connected to the server disconnection queue.
    pub(super) fn new(disconnected_clients: Arc<Mutex<Vec<ClientId>>>) -> Self {
        Self {
            compositor_state: CompositorClientState::default(),
            disconnected_clients,
        }
    }
}

impl ClientData for AureaClientState {
    fn initialized(&self, _client_id: ClientId) {}

    fn disconnected(&self, client_id: ClientId, _reason: DisconnectReason) {
        let mut disconnected_clients = self
            .disconnected_clients
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        disconnected_clients.push(client_id);
    }
}

impl CompositorHandler for AureaCompositor {
    fn compositor_state(&mut self) -> &mut CompositorState {
        self.compositor_state()
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client
            .get_data::<AureaClientState>()
            .expect("AureaClientState must be attached to every client")
            .compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        // Smithay 0.7 exposes commit and destruction hooks, but no surface-
        // creation callback. Count every surface at its first commit and
        // release quota from the destruction hook (also called on disconnect).
        let surface_id = surface.id();
        if !self.has_tracked_surface(&surface_id) {
            let client_id = surface.client().id();
            if !self
                .register_surface(surface_id, client_id.clone())
            {
                self.display_handle()
                    .backend_handle()
                    .kill_client(client_id, DisconnectReason::ConnectionClosed);
                return;
            }

            add_destruction_hook(surface, |state: &mut AureaCompositor, surface| {
                state.unregister_surface(&surface.id());
            });
        }
        let mapped = with_states(surface, |states| {
            let mut cached_state = states.cached_state.get::<SurfaceAttributes>();
            let current = cached_state.current();

            match current.buffer.as_ref() {
                Some(BufferAssignment::NewBuffer(_)) => Some(true),
                Some(BufferAssignment::Removed) => Some(false),
                None => None,
            }
        });

        if let Some(mapped) = self.window_manager().update_surface_buffer(surface, mapped) {
            if mapped {
                self.focus_surface_if_unset(surface.clone());
            } else {
                let next_focus = self.window_manager().active_workspace_surface();
                self.focus_surface(next_focus);
            }
        }

        self.request_all_output_frames();
    }
}

delegate_compositor!(AureaCompositor);
