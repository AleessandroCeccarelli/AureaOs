// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Core Wayland compositor state.

use std::{collections::BTreeMap, error::Error};

use smithay::{
    input::{SeatHandler, SeatState},
    output::Output,
    reexports::wayland_server::{DisplayHandle, protocol::wl_surface::WlSurface},
    wayland::{compositor::CompositorState, shell::xdg::XdgShellState},
};

use crate::{
    output::{
        OutputId,
        manager::{OutputManager, OutputReconciliation},
        wayland::{OutputPublicationError, publish_output, publish_outputs},
    },
    render::RenderScene,
    renderer::{RenderBackend, RenderTarget},
    window_manager::WindowManager,
};

/// Errors returned while rendering a compositor output.
#[derive(Debug)]
pub enum CompositorRenderError<E> {
    /// The requested output is not registered.
    UnknownOutput(OutputId),

    /// The requested output is disabled.
    OutputDisabled(OutputId),

    /// The rendering backend failed.
    Backend(E),
}

impl<E: Error + 'static> std::fmt::Display for CompositorRenderError<E> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownOutput(output_id) => {
                write!(formatter, "Unknown output: {output_id}")
            }
            Self::OutputDisabled(output_id) => {
                write!(formatter, "Output {output_id} is disabled")
            }
            Self::Backend(error) => {
                write!(formatter, "Rendering backend failed: {error}")
            }
        }
    }
}

impl<E: Error + 'static> Error for CompositorRenderError<E> {}

/// Failure while publishing a committed output reconciliation.
#[derive(Debug)]
pub struct OutputReconciliationError {
    output_id: OutputId,
    source: OutputPublicationError,
    rollback_error: Option<OutputPublicationError>,
}

impl std::fmt::Display for OutputReconciliationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "Failed to publish reconciled output {}: {}",
            self.output_id, self.source
        )?;
        if let Some(error) = &self.rollback_error {
            write!(
                formatter,
                "; restoring prior output globals also failed: {error}"
            )?;
        }
        Ok(())
    }
}

impl Error for OutputReconciliationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

impl SeatHandler for AureaCompositor {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }
}

/// Core state owned by the AureaOS compositor.
pub struct AureaCompositor {
    compositor_state: CompositorState,
    xdg_shell_state: XdgShellState,
    seat_state: SeatState<Self>,
    window_manager: WindowManager,
    output_manager: OutputManager,
    wayland_outputs: BTreeMap<OutputId, Output>,
}

impl AureaCompositor {
    /// Creates the compositor protocol states and managers.
    pub fn new(display: &DisplayHandle) -> Self {
        let compositor_state = CompositorState::new::<Self>(display);
        let xdg_shell_state = XdgShellState::new::<Self>(display);

        Self {
            compositor_state,
            xdg_shell_state,
            seat_state: SeatState::new(),
            window_manager: WindowManager::new(),
            output_manager: OutputManager::new(),
            wayland_outputs: BTreeMap::new(),
        }
    }

    /// Publishes all registered outputs as Wayland globals.
    ///
    /// All outputs are validated before their globals are created.
    /// Existing Wayland outputs are replaced only after publication succeeds.
    pub fn publish_wayland_outputs(
        &mut self,
        display: &DisplayHandle,
    ) -> Result<(), OutputPublicationError> {
        let outputs = publish_outputs(display, &self.output_manager)?;

        let output_ids = self
            .output_manager
            .registry()
            .iter()
            .filter(|output| output.is_enabled())
            .map(|output| output.id());

        self.wayland_outputs = output_ids.zip(outputs).collect();

        Ok(())
    }

    /// Publishes one registered output as a Wayland global.
    ///
    /// An already published output is left unchanged.
    /// Disabled outputs are not published.
    pub fn publish_wayland_output(
        &mut self,
        display: &DisplayHandle,
        output_id: OutputId,
    ) -> Result<(), OutputPublicationError> {
        if self.wayland_outputs.contains_key(&output_id) {
            return Ok(());
        }

        let Some(output) = self.output_manager.registry().get(output_id) else {
            return Ok(());
        };

        let position = self
            .output_manager
            .layout()
            .get(output_id)
            .unwrap_or_default();

        if let Some(wayland_output) = publish_output(display, output, position)? {
            self.wayland_outputs.insert(output_id, wayland_output);
        }

        Ok(())
    }

    /// Republishes one output after its description or position changed.
    /// The replacement is fully validated and created before the retained
    /// output handle is swapped, so a publication error leaves the old global.
    pub fn refresh_wayland_output(
        &mut self,
        display: &DisplayHandle,
        output_id: OutputId,
    ) -> Result<(), OutputPublicationError> {
        let Some(output) = self.output_manager.registry().get(output_id) else {
            self.wayland_outputs.remove(&output_id);
            return Ok(());
        };
        if !output.is_enabled() {
            self.wayland_outputs.remove(&output_id);
            return Ok(());
        }
        let position = self
            .output_manager
            .layout()
            .get(output_id)
            .unwrap_or_default();
        if let Some(replacement) = publish_output(display, output, position)? {
            self.wayland_outputs.insert(output_id, replacement);
        } else {
            self.wayland_outputs.remove(&output_id);
        }
        Ok(())
    }

    /// Publishes a per-device model reconciliation, rolling back the output
    /// manager and affected Wayland globals if any changed global is invalid.
    pub fn publish_output_reconciliation(
        &mut self,
        display: &DisplayHandle,
        previous: OutputManager,
        changes: &OutputReconciliation,
    ) -> Result<(), OutputReconciliationError> {
        let mut affected = changes.added.clone();
        affected.extend(changes.updated.iter().copied());
        affected.extend(changes.removed.iter().copied());

        for id in &changes.removed {
            self.unpublish_wayland_output(*id);
        }
        for id in changes.added.iter().chain(&changes.updated) {
            if let Err(source) = self.refresh_wayland_output(display, *id) {
                self.output_manager = previous.clone();
                let mut rollback_error = None;
                for rollback_id in &affected {
                    if previous.registry().get(*rollback_id).is_some() {
                        if let Err(error) = self.refresh_wayland_output(display, *rollback_id) {
                            rollback_error.get_or_insert(error);
                        }
                    } else {
                        self.unpublish_wayland_output(*rollback_id);
                    }
                }
                return Err(OutputReconciliationError {
                    output_id: *id,
                    source,
                    rollback_error,
                });
            }
        }
        Ok(())
    }

    /// Removes one published Wayland output.
    ///
    /// Dropping the retained Smithay output removes its global.
    /// Returns whether an output was actually removed.
    pub fn unpublish_wayland_output(&mut self, output_id: OutputId) -> bool {
        self.wayland_outputs.remove(&output_id).is_some()
    }

    /// Returns the Smithay compositor protocol state.
    pub fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    /// Returns the XDG shell protocol state.
    pub fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    /// Returns the window manager.
    pub fn window_manager(&mut self) -> &mut WindowManager {
        &mut self.window_manager
    }

    /// Returns the output manager.
    pub fn output_manager(&mut self) -> &mut OutputManager {
        &mut self.output_manager
    }

    /// Builds a snapshot of the currently mapped surfaces.
    pub fn render_scene(&self) -> RenderScene<'_> {
        RenderScene::from_window_manager(&self.window_manager)
    }

    /// Renders the current scene on the requested enabled output.
    ///
    /// The backend owns graphics-API details and is responsible for
    /// presenting the completed frame.
    pub fn render_output<B: RenderBackend>(
        &self,
        backend: &mut B,
        output_id: OutputId,
    ) -> Result<(), CompositorRenderError<B::Error>> {
        let output = self
            .output_manager
            .registry()
            .get(output_id)
            .ok_or(CompositorRenderError::UnknownOutput(output_id))?;

        if !output.is_enabled() {
            return Err(CompositorRenderError::OutputDisabled(output_id));
        }

        let target = RenderTarget::from_output(output)
            .ok_or(CompositorRenderError::OutputDisabled(output_id))?;

        let scene = self.render_scene();

        backend
            .render_frame(&target, &scene)
            .map_err(CompositorRenderError::Backend)
    }
}

#[cfg(test)]
#[path = "../tests/unit/compositor_output_reconciliation.rs"]
mod output_reconciliation_tests;
