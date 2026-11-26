// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Shared contracts between AureaOS desktop components.
//!
//! This crate contains technology-independent interfaces.
//! It must not depend on the shell, compositor or graphics backends.

pub mod appearance;
pub mod calendar;
pub mod desktop_interaction;
pub mod desktop;
pub mod display;
pub mod hardware_monitor;
pub mod input;
pub mod keyboard;
pub mod notes;
pub mod notifications;
pub mod time;

use std::io;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use desktop_interaction::DesktopInteractionSender;
use display::DisplayColorController;

/// Shared shutdown signal used to coordinate desktop components.
#[derive(Clone, Default)]
pub struct ShutdownSignal {
    requested: Arc<AtomicBool>,
}

impl ShutdownSignal {
    /// Creates a new shutdown signal in the inactive state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Requests the component to shut down.
    pub fn request_shutdown(&self) {
        self.requested.store(true, Ordering::Release);
    }

    /// Returns whether shutdown has been requested.
    pub fn is_shutdown_requested(&self) -> bool {
        self.requested.load(Ordering::Acquire)
    }

    /// Returns a shared handle to the underlying atomic shutdown flag.
    pub fn atomic_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.requested)
    }
}

/// A running compositor service.
///
/// The service is created and executed on its owning thread.
/// It does not need to be transferable between threads.
pub trait CompositorService: 'static {
    /// Runs the compositor until shutdown is requested or an error occurs.
    fn run(&mut self, shutdown: &ShutdownSignal) -> io::Result<()>;
}

/// Creates a compositor service.
///
/// The factory can be transferred to the compositor worker thread.
/// The service itself is created inside that thread. The display controller
/// is shared with the shell and remains valid for the service lifetime.
pub trait CompositorFactory: Send + Sync + 'static {
    /// Initializes a compositor service with its shared display policy channel.
    fn create(
        &self,
        display_color: DisplayColorController,
        interactions: DesktopInteractionSender,
        touchpad_settings: std::sync::mpsc::Receiver<input::TouchpadSettingsCommand>,
        pointer_settings: std::sync::mpsc::Receiver<input::PointerSettingsCommand>,
        desktop_pointer_behavior: std::sync::mpsc::Receiver<input::DesktopPointerBehaviorCommand>,
    ) -> io::Result<Box<dyn CompositorService>>;
}
