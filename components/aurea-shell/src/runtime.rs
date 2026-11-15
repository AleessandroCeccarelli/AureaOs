// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Shell runtime management.

use std::sync::mpsc::Receiver;

use aurea_desktop_api::display::{DisplayColorAdjustment, DisplayColorController};

use crate::desktop::{DesktopState, NightLightState};
use crate::dispatcher::{self, RuntimeCommand};
use crate::error::ShellError;
use crate::event::ShellEvent;
use crate::session::{DesktopSession, SessionState};
use crate::state::{ShellLifecycle, ShellState};

/// Owns the runtime and desktop session lifecycles.
pub struct ShellRuntime {
    lifecycle: ShellLifecycle,
    session: DesktopSession,
    desktop: DesktopState,
    display_color: DisplayColorController,
}

impl ShellRuntime {
    /// Creates a runtime that has not been started yet.
    pub fn new() -> Self {
        Self::with_display_color(DisplayColorController::new())
    }

    /// Creates a runtime connected to the compositor's shared display policy.
    pub fn with_display_color(display_color: DisplayColorController) -> Self {
        Self {
            lifecycle: ShellLifecycle::new(),
            session: DesktopSession::new(),
            desktop: DesktopState::default(),
            display_color,
        }
    }

    /// Publishes the effective Night Light preference to the compositor.
    ///
    /// The scheduler supplies whether the selected schedule is currently
    /// active. Manual mode ignores that value.
    pub fn update_night_light(&mut self, schedule_active: bool) {
        let adjustment = self
            .desktop
            .night_light
            .display_adjustment(schedule_active);
        self.display_color.set_adjustment(adjustment);
    }

    /// Sets Night Light preferences and publishes the resulting display policy.
    pub fn set_night_light(&mut self, state: NightLightState, schedule_active: bool) {
        self.desktop.night_light = state;
        self.update_night_light(schedule_active);
    }

    /// Returns the latest effective display adjustment.
    pub fn display_color_adjustment(&self) -> DisplayColorAdjustment {
        self.display_color.adjustment()
    }

    /// Starts the runtime and activates the desktop session.
    ///
    /// If session activation fails, the runtime lifecycle is rolled back.
    pub fn start(&mut self) -> Result<(), ShellError> {
        self.lifecycle.start()?;

        if let Err(error) = self.session.activate() {
            self.lifecycle.stop()?;
            return Err(error);
        }

        Ok(())
    }

    /// Runs the event loop until shutdown is requested or the channel closes.
    pub fn run_event_loop(&mut self, receiver: Receiver<ShellEvent>) -> Result<(), ShellError> {
        while self.state() == ShellState::Running {
            match receiver.recv() {
                Ok(event) => self.handle_event(event)?,
                Err(_) => self.stop()?,
            }
        }

        Ok(())
    }

    /// Dispatches an event and executes the resulting runtime command.
    pub fn handle_event(&mut self, event: ShellEvent) -> Result<(), ShellError> {
        match dispatcher::dispatch(event) {
            RuntimeCommand::LockSession => self.session.lock(),
            RuntimeCommand::UnlockSession => self.session.unlock(),
            RuntimeCommand::Shutdown => self.stop(),
        }
    }

    /// Stops the desktop session and then the runtime.
    pub fn stop(&mut self) -> Result<(), ShellError> {
        match self.session.state() {
            SessionState::Active | SessionState::Locked => {
                self.session.stop()?;
                self.session.finish_stop()?;
            }
            SessionState::Stopping => {
                self.session.finish_stop()?;
            }
            SessionState::Stopped => {}
            SessionState::Starting => {
                return Err(ShellError::InvalidLifecycleTransition(
                    "Session cannot stop before activation",
                ));
            }
        }

        self.lifecycle.stop()
    }

    /// Returns the current desktop presentation state.
    pub fn desktop_state(&self) -> &DesktopState {
        &self.desktop
    }

    /// Returns mutable desktop presentation state.
    ///
    /// Call update_night_light after changing Night Light preferences.
    pub fn desktop_state_mut(&mut self) -> &mut DesktopState {
        &mut self.desktop
    }

    /// Returns the current runtime state.
    pub fn state(&self) -> ShellState {
        self.lifecycle.state()
    }

    /// Returns the current desktop session state.
    pub fn session_state(&self) -> SessionState {
        self.session.state()
    }
}

impl Default for ShellRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "../tests/unit/runtime.rs"]
mod tests;
