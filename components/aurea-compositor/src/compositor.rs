// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Core Wayland compositor state.

use std::error::Error;

use smithay::{
    output::Output,
    reexports::wayland_server::DisplayHandle,
    wayland::{
        compositor::CompositorState,
        shell::xdg::XdgShellState,
    },
};

use crate::{
    output::{
        manager::OutputManager,
        wayland::{publish_outputs, OutputPublicationError},
        OutputId,
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

/// Core state owned by the AureaOS compositor.
pub struct AureaCompositor {
    compositor_state: CompositorState,
    xdg_shell_state: XdgShellState,
    window_manager: WindowManager,
    output_manager: OutputManager,
    wayland_outputs: Vec<Output>,
}

impl AureaCompositor {
    /// Creates the compositor protocol states and managers.
    pub fn new(display: &DisplayHandle) -> Self {
        let compositor_state = CompositorState::new::<Self>(display);
        let xdg_shell_state = XdgShellState::new::<Self>(display);

        Self {
            compositor_state,
            xdg_shell_state,
            window_manager: WindowManager::new(),
            output_manager: OutputManager::new(),
            wayland_outputs: Vec::new(),
        }
    }

    /// Publishes registered outputs as Wayland globals.
    ///
    /// Output validation errors are returned to the caller instead of
    /// silently producing an incomplete or invalid Wayland configuration.
    pub fn publish_wayland_outputs(
        &mut self,
        display: &DisplayHandle,
    ) -> Result<(), OutputPublicationError> {
        self.wayland_outputs = publish_outputs(display, &self.output_manager)?;
        Ok(())
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
