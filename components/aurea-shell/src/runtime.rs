// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Shell runtime management.

use std::{
    collections::VecDeque,
    sync::mpsc::{Receiver, RecvTimeoutError},
    time::Duration,
};

use aurea_desktop_api::desktop_interaction::{
    DesktopInteraction, DesktopInteractionReceiver, DesktopSelectionController,
};
use aurea_desktop_api::input::{
    InputDeviceId, TouchpadSettings, TouchpadSettingsController, touchpad_settings_channel,
};

use aurea_desktop_api::display::{DisplayColorAdjustment, DisplayColorController};

use crate::desktop::{DesktopPanel, DesktopState, NightLightState};
use crate::dispatcher::{self, RuntimeCommand};
use crate::error::ShellError;
use crate::event::ShellEvent;
use crate::session::{DesktopSession, SessionState};
use crate::state::{ShellLifecycle, ShellState};

/// Maximum number of semantic interactions retained for the shell consumer.
const MAX_PENDING_DESKTOP_INTERACTIONS: usize = 32;

/// Owns the runtime and desktop session lifecycles.
pub struct ShellRuntime {
    lifecycle: ShellLifecycle,
    session: DesktopSession,
    desktop: DesktopState,
    display_color: DisplayColorController,
    touchpad_settings: TouchpadSettingsController,
    pending_desktop_interactions: VecDeque<DesktopInteraction>,
    desktop_selection: DesktopSelectionController,
}

impl ShellRuntime {
    /// Creates a runtime that has not been started yet.
    pub fn new() -> Self {
        Self::with_display_color(DisplayColorController::new())
    }

    /// Creates a runtime connected to the compositor's shared display policy.
    pub fn with_display_color(display_color: DisplayColorController) -> Self {
        let (touchpad_settings, _receiver) = touchpad_settings_channel();
        Self::with_controllers(display_color, touchpad_settings)
    }

    /// Creates a runtime with shared display and touchpad policy channels.
    pub fn with_controllers(
        display_color: DisplayColorController,
        touchpad_settings: TouchpadSettingsController,
    ) -> Self {
        Self {
            lifecycle: ShellLifecycle::new(),
            session: DesktopSession::new(),
            desktop: DesktopState::default(),
            display_color,
            touchpad_settings,
            pending_desktop_interactions: VecDeque::new(),
            desktop_selection: DesktopSelectionController::default(),
        }
    }

    /// Publishes the effective Night Light preference to the compositor.
    ///
    /// The scheduler supplies whether the selected schedule is currently
    /// active. Manual mode ignores that value.
    pub fn update_night_light(&mut self, schedule_active: bool) {
        let adjustment = self.desktop.night_light.display_adjustment(schedule_active);
        self.display_color.set_adjustment(adjustment);
    }

    /// Sets Night Light preferences and publishes the resulting display policy.
    pub fn set_night_light(&mut self, state: NightLightState, schedule_active: bool) {
        self.desktop.night_light = state;
        self.update_night_light(schedule_active);
    }

    /// Requests a live profile update for one connected touchpad.
    pub fn set_touchpad_settings(&self, device: InputDeviceId, settings: TouchpadSettings) -> bool {
        self.touchpad_settings.set(device, settings).is_ok()
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
        let (_sender, interactions) =
            aurea_desktop_api::desktop_interaction::desktop_interaction_channel();
        self.run_event_loop_with_interactions(receiver, interactions)
    }

    /// Runs the shell loop while also receiving compositor-originated actions.
    pub fn run_event_loop_with_interactions(
        &mut self,
        receiver: Receiver<ShellEvent>,
        interactions: DesktopInteractionReceiver,
    ) -> Result<(), ShellError> {
        self.desktop_selection = interactions.selection_controller();

        while self.state() == ShellState::Running {
            match receiver.recv_timeout(Duration::from_millis(50)) {
                Ok(event) => self.handle_event(event)?,
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => self.stop()?,
            }

            while let Ok(interaction) = interactions.try_recv() {
                self.handle_desktop_interaction(interaction);
            }
        }

        Ok(())
    }

    /// Applies shell-owned state changes and retains presentation actions.
    ///
    /// Workspace changes arrive with the compositor's authoritative index,
    /// regardless of whether a gesture or keyboard shortcut initiated them.
    pub fn handle_desktop_interaction(&mut self, interaction: DesktopInteraction) {
        match interaction {
            DesktopInteraction::WorkspaceChanged { index } => {
                self.desktop.workspaces.switch_to(index);
            }
            DesktopInteraction::ShowOverview => {
                self.desktop.panels.show(DesktopPanel::Overview);
            }
            DesktopInteraction::ShowContextMenu => {
                self.desktop.panels.show(DesktopPanel::ContextMenu);
            }
            DesktopInteraction::DesktopItemSelected { ref id } => {
                self.desktop.select_item(id.clone());
                self.desktop_selection.set_selected_item(Some(id.clone()));
            }
            DesktopInteraction::DesktopSelectionCleared => {
                self.desktop.clear_item_selection();
                self.desktop_selection.set_selected_item(None);
            }
            DesktopInteraction::DesktopItemActivated { .. } => {
                // Activation is retained for the desktop/file service consumer.
                // Launching is deliberately not inferred by the shell runtime.
            }
        }

        if self.pending_desktop_interactions.len() == MAX_PENDING_DESKTOP_INTERACTIONS {
            self.pending_desktop_interactions.pop_front();
        }
        self.pending_desktop_interactions.push_back(interaction);
    }

    /// Retrieves the next semantic desktop action received from the compositor.
    pub fn take_desktop_interaction(&mut self) -> Option<DesktopInteraction> {
        self.pending_desktop_interactions.pop_front()
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

    /// Updates the desktop Trash artwork state from the system Trash service.
    ///
    /// The caller owns filesystem monitoring and maps its service snapshot to
    /// the shell's semantic Empty/PartiallyFull/Full state.
    pub fn set_trash_contents(&mut self, state: crate::desktop::TrashContentsState) {
        self.desktop.set_trash_contents(state);
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

    /// Closes any transient desktop surface currently requested.
    pub fn dismiss_desktop_panel(&mut self) {
        self.desktop.panels.close();
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
