
 // SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Wayland server lifecycle and client dispatch.

use std::io;
use std::os::fd::AsFd;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use smithay::{
    delegate_compositor,
    reexports::{
        calloop::{
            EventLoop, Interest, Mode,
            generic::Generic,
        },
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            protocol::wl_surface::WlSurface,
            Client, Display,
        },
    },
    wayland::{
        compositor::{
            BufferAssignment, CompositorClientState, CompositorHandler,
            CompositorState, SurfaceAttributes, with_states,
        },
        socket::ListeningSocketSource,
    },
};

#[cfg(feature = "drm")]
use smithay::backend::session::{Event as SessionEvent, Session};

use crate::AureaCompositor;

#[cfg(feature = "drm")]
use crate::drm::DrmSession;

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

    fn commit(&mut self, surface: &WlSurface) {
        let mapped = with_states(surface, |states| {
            let current = states
                .cached_state
                .get::<SurfaceAttributes>()
                .current();

            match current.buffer.as_ref() {
                Some(BufferAssignment::NewBuffer(_)) => Some(true),
                Some(BufferAssignment::Removed) => Some(false),
                None => None,
            }
        });

        self.window_manager()
            .update_surface_buffer(surface, mapped);
    }
}

delegate_compositor!(AureaCompositor);

#[cfg(feature = "drm")]
type OptionalDrmSession = Option<DrmSession>;

#[cfg(not(feature = "drm"))]
type OptionalDrmSession = ();

/// State accessed by Calloop event callbacks.
struct WaylandState {
    display: Display<AureaCompositor>,
    compositor: AureaCompositor,
    clients: Vec<Client>,
    error: Option<io::Error>,

    #[cfg(feature = "drm")]
    _drm_session: Option<DrmSession>,

    #[cfg(feature = "drm")]
    session_active: bool,
}

/// Owns the Wayland display and its Calloop event loop.
pub struct WaylandServer {
    event_loop: EventLoop<WaylandState>,
    state: WaylandState,
}

impl WaylandServer {
    /// Creates a server bound to the requested Wayland socket name.
    ///
    /// This constructor does not require a native DRM session.
    pub fn new(socket_name: &str) -> io::Result<Self> {
        #[cfg(feature = "drm")]
        {
            Self::build(socket_name, None)
        }

        #[cfg(not(feature = "drm"))]
        {
            Self::build(socket_name, ())
        }
    }

    /// Creates a server with a native libseat session.
    #[cfg(feature = "drm")]
    pub fn new_with_drm(socket_name: &str) -> io::Result<Self> {
        let session = DrmSession::new().map_err(io::Error::other)?;
        Self::build(socket_name, Some(session))
    }

    /// Returns whether the native session is currently active.
    #[cfg(feature = "drm")]
    pub fn session_active(&self) -> bool {
        self.state.session_active
    }

    fn build(
        socket_name: &str,
        #[cfg(feature = "drm")] drm_session: OptionalDrmSession,
        #[cfg(not(feature = "drm"))] _drm_session: OptionalDrmSession,
    ) -> io::Result<Self> {
        let mut event_loop = EventLoop::<WaylandState>::try_new()
            .map_err(io::Error::other)?;

        let mut display = Display::<AureaCompositor>::new()?;
        let display_handle = display.handle();
        let compositor = AureaCompositor::new(&display_handle);

        #[cfg(feature = "drm")]
        let session_active = drm_session
            .as_ref()
            .map(|session| session.session().is_active())
            .unwrap_or(true);

        let mut state = WaylandState {
            display,
            compositor,
            clients: Vec::new(),
            error: None,

            #[cfg(feature = "drm")]
            _drm_session: drm_session,

            #[cfg(feature = "drm")]
            session_active,
        };

        let loop_handle = event_loop.handle();

        // Accept new Wayland clients when the listening socket becomes readable.
        let listening_socket = ListeningSocketSource::with_name(socket_name)
            .map_err(io::Error::other)?;

        loop_handle
            .insert_source(listening_socket, |stream, _, state| {
                match state
                    .display
                    .handle()
                    .insert_client(stream, Arc::new(AureaClientState::default()))
                {
                    Ok(client) => state.clients.push(client),
                    Err(error) => {
                        state.error = Some(io::Error::other(error));
                    }
                }
            })
            .map_err(io::Error::other)?;

        // Monitor native session activation and pause events.
        #[cfg(feature = "drm")]
        if let Some(session) = state._drm_session.as_mut() {
            let notifier = session
                .take_notifier()
                .map_err(io::Error::other)?;

            loop_handle
                .insert_source(notifier, |event, _, state| {
                    match event {
                        SessionEvent::ActivateSession => {
                            state.session_active = true;
                        }
                        SessionEvent::PauseSession => {
                            state.session_active = false;
                        }
                    }
                })
                .map_err(io::Error::other)?;
        }

        // Monitor the Wayland display fd for incoming client requests.
        let display_fd = state
            .display
            .backend()
            .poll_fd()
            .as_fd()
            .try_clone_to_owned()?;

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

                    if let Err(error) = state.display.flush_clients() {
                        state.error = Some(io::Error::other(error));
                        stop_handle.stop();
                    }
                },
            )
            .map_err(io::Error::other)?;

        Ok(Self { event_loop, state })
    }

    /// Runs the server until shutdown is requested.
    pub fn run(&mut self, shutdown: &AtomicBool) -> io::Result<()> {
        while !shutdown.load(Ordering::Acquire) {
            self.event_loop
                .dispatch(Duration::from_millis(100), &mut self.state)
                .map_err(io::Error::other)?;

            if let Some(error) = self.state.error.take() {
                return Err(error);
            }
        }

        Ok(())
    }
}
