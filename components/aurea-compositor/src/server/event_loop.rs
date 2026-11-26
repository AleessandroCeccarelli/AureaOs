// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Wayland server runtime and Calloop event loop execution.

use std::{
    io,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use aurea_desktop_api::{
    desktop_interaction::{DesktopInteractionSender, desktop_interaction_channel},
    display::DisplayColorController,
    input::{DesktopPointerBehaviorCommand, PointerSettingsCommand, TouchpadSettingsCommand, desktop_pointer_behavior_channel, pointer_settings_channel, touchpad_settings_channel},
    keyboard::KeyboardLayoutId,
};
use smithay::reexports::calloop::EventLoop;
use std::sync::mpsc::Receiver;

#[cfg(feature = "drm")]
use crate::drm::{DrmSession, device::DrmDevice};

use super::state::WaylandState;

#[cfg(feature = "drm")]
use super::session;

/// Owns the Wayland display state and its Calloop event loop.
pub struct WaylandServer {
    event_loop: EventLoop<'static, WaylandState>,
    state: WaylandState,
}

impl WaylandServer {
    /// Creates a server bound to the requested Wayland socket name.
    ///
    /// This constructor does not require a native DRM session.
    pub fn new(socket_name: &str) -> io::Result<Self> {
        Self::new_with_display_color(socket_name, DisplayColorController::new())
    }

    /// Creates a server with a shared display color policy channel.
    pub fn new_with_display_color(
        socket_name: &str,
        display_color: DisplayColorController,
    ) -> io::Result<Self> {
        let (interactions, _receiver) = desktop_interaction_channel();
        let (_touchpad_controller, touchpad_settings) = touchpad_settings_channel();
        let (_pointer_controller, pointer_settings) = pointer_settings_channel();
        let (_desktop_pointer_behavior_controller, desktop_pointer_behavior) = desktop_pointer_behavior_channel();
        #[cfg(feature = "drm")]
        let parts = super::builder::build(
            socket_name,
            None,
            None,
            display_color,
            interactions,
            touchpad_settings,
            pointer_settings,
            desktop_pointer_behavior,
        );

        #[cfg(not(feature = "drm"))]
        let parts =
            super::builder::build(socket_name, display_color, interactions, touchpad_settings, pointer_settings, desktop_pointer_behavior);

        let (event_loop, state) = parts?;

        Ok(Self { event_loop, state })
    }

    /// Creates a server with a native libseat session.
    #[cfg(feature = "drm")]
    pub fn new_with_drm(socket_name: &str) -> io::Result<Self> {
        let drm_session = session::create_session()?;
        let (interactions, _receiver) = desktop_interaction_channel();
        let (_touchpad_controller, touchpad_settings) = touchpad_settings_channel();
        let (_pointer_controller, pointer_settings) = pointer_settings_channel();
        let (_desktop_pointer_behavior_controller, desktop_pointer_behavior) = desktop_pointer_behavior_channel();
        Self::build(
            socket_name,
            Some(drm_session),
            None,
            DisplayColorController::new(),
            interactions,
            touchpad_settings,
            pointer_settings,
            desktop_pointer_behavior,
        )
    }

    /// Creates a server with a libseat session and the primary GPU for a seat.
    #[cfg(feature = "drm")]
    pub fn new_with_primary_drm(
        socket_name: &str,
        seat: &str,
        display_color: DisplayColorController,
        interactions: DesktopInteractionSender,
        touchpad_settings: Receiver<TouchpadSettingsCommand>,
        pointer_settings: Receiver<PointerSettingsCommand>,
        desktop_pointer_behavior: Receiver<DesktopPointerBehaviorCommand>,
    ) -> io::Result<Self> {
        let mut drm_session = session::create_session()?;
        let drm_device = session::open_primary_device(&mut drm_session, seat)?;

        Self::build(
            socket_name,
            Some(drm_session),
            Some(drm_device),
            display_color,
            interactions,
            touchpad_settings,
            pointer_settings,
        )
    }

    /// Creates a server with a native libseat session and an opened DRM device.
    #[cfg(feature = "drm")]
    pub fn new_with_drm_device(socket_name: &str, device_path: &str) -> io::Result<Self> {
        let mut drm_session = session::create_session()?;
        let drm_device = session::open_device(&mut drm_session, device_path)?;

        let (interactions, _receiver) = desktop_interaction_channel();
        let (_touchpad_controller, touchpad_settings) = touchpad_settings_channel();
        let (_pointer_controller, pointer_settings) = pointer_settings_channel();
        Self::build(
            socket_name,
            Some(drm_session),
            Some(drm_device),
            DisplayColorController::new(),
            interactions,
            touchpad_settings,
            pointer_settings,
        )
    }

    /// Returns whether the native session is currently active.
    #[cfg(feature = "drm")]
    pub fn session_active(&self) -> bool {
        self.state.session_active
    }

    #[cfg(feature = "drm")]
    fn build(
        socket_name: &str,
        drm_session: Option<DrmSession>,
        drm_device: Option<DrmDevice>,
        display_color: DisplayColorController,
        interactions: DesktopInteractionSender,
        touchpad_settings: Receiver<TouchpadSettingsCommand>,
        pointer_settings: Receiver<PointerSettingsCommand>,
        desktop_pointer_behavior: Receiver<DesktopPointerBehaviorCommand>,
    ) -> io::Result<Self> {
        let (event_loop, state) = super::builder::build(
            socket_name,
            drm_session,
            drm_device,
            display_color,
            interactions,
            touchpad_settings,
            pointer_settings,
            desktop_pointer_behavior,
        )?;

        Ok(Self { event_loop, state })
    }

    /// Applies the user's selected keyboard layout before the event loop starts.
    pub fn set_keyboard_layout(&mut self, layout: &KeyboardLayoutId) -> io::Result<()> {
        self.state.compositor.set_keyboard_layout(layout)
    }

    /// Sets the initial desktop Trash artwork state before the event loop starts.
    pub fn set_trash_state(&mut self, state: aurea_desktop_api::desktop::DesktopTrashState) {
        self.state.compositor.desktop_items_mut().set_trash_state(state);
    }

    /// Runs the server until shutdown is requested.
    ///
    /// KMS outputs are explicitly disabled before their GBM/Vulkan resources
    /// are released, both on normal shutdown and when the event loop fails.
    pub fn run(&mut self, shutdown: &AtomicBool) -> io::Result<()> {
        eprintln!(
            "AureaOS compositor: entering Calloop event loop; shutdown_requested={}",
            shutdown.load(Ordering::Acquire)
        );
        #[cfg(feature = "drm")]
        {
            eprintln!(
                "AureaOS compositor: session_active={} discovered_drm_devices={} output_surface_managers={} vulkan_backends={}",
                self.state.session_active,
                self.state.udev_device_ids.len(),
                self.state.drm_surfaces.len(),
                self.state.vulkan_backends.len()
            );
            self.state.compositor.request_all_output_frames();
        }

        let run_result = (|| {
            while !shutdown.load(Ordering::Acquire) {
                #[cfg(feature = "drm")]
                let animation_active = self.state.compositor.cursor_animation_active()
                    || self.state.vulkan_backends.values().any(|backend| {
                        backend.has_pending_native_cursor_work()
                    });

                #[cfg(not(feature = "drm"))]
                let animation_active = false;

                #[cfg(feature = "drm")]
                if animation_active {
                    self.state.compositor.request_all_output_frames();
                }

                let dispatch_interval = if animation_active {
                    Duration::from_millis(16)
                } else {
                    Duration::from_millis(100)
                };
                self.event_loop
                    .dispatch(dispatch_interval, &mut self.state)
                    .map_err(io::Error::other)?;

                #[cfg(feature = "drm")]
                self.state.compositor.apply_deferred_pointer_recenter();

                #[cfg(feature = "drm")]
                super::input::apply_pending_touchpad_settings(&mut self.state);
                #[cfg(feature = "drm")]
                super::input::apply_pending_pointer_settings(&mut self.state);
                #[cfg(feature = "drm")]
                super::input::apply_pending_desktop_pointer_behavior(&mut self.state);

                #[cfg(feature = "drm")]
                super::rendering::render_pending_outputs(&mut self.state)?;

                if let Some(error) = self.state.error.take() {
                    return Err(error);
                }
            }

            Ok(())
        })();

        #[cfg(feature = "drm")]
        let cleanup_result = match super::session_events::clear_drm_surfaces(&mut self.state) {
            None => {
                super::gbm_lifecycle::clear_gbm_devices(&mut self.state);
                self.state.vulkan_backends.clear();
                Ok(())
            }
            Some(error) => Err(io::Error::other(format!(
                "{error}; dependent GBM/Vulkan resources were retained"
            ))),
        };

        #[cfg(not(feature = "drm"))]
        let cleanup_result: io::Result<()> = Ok(());

        if run_result.is_ok() {
            eprintln!(
                "AureaOS compositor: event loop stopped cleanly; DRM/GBM/Vulkan cleanup completed={}",
                cleanup_result.is_ok()
            );
        } else if let Err(error) = &run_result {
            eprintln!("AureaOS compositor: event loop failed: {error}");
        }

        match (run_result, cleanup_result) {
            (Err(primary), Err(cleanup)) => {
                eprintln!("AureaOS: graphics cleanup also failed during shutdown: {cleanup}");
                Err(primary)
            }
            (Err(primary), Ok(())) => Err(primary),
            (Ok(()), Err(cleanup)) => Err(cleanup),
            (Ok(()), Ok(())) => Ok(()),
        }
    }
}
