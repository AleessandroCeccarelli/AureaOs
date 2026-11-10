
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Scene preparation and output rendering coordination.

use std::error::Error;

use crate::{
    frame::FrameLifecycleError,
    output::OutputId,
    render::RenderScene,
    renderer::{RenderBackend, RenderTarget},
};

use super::AureaCompositor;

/// Errors returned while rendering a compositor output.
#[derive(Debug)]
pub enum CompositorRenderError<E> {
    /// The requested output is not registered.
    UnknownOutput(OutputId),

    /// The requested output is disabled.
    OutputDisabled(OutputId),

    /// No frame request is waiting for this output.
    FrameNotRequested(OutputId),

    /// The frame lifecycle could not be completed.
    FrameLifecycle(FrameLifecycleError),

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
            Self::FrameNotRequested(output_id) => {
                write!(formatter, "No frame was requested for output {output_id}")
            }
            Self::FrameLifecycle(error) => {
                write!(formatter, "Frame lifecycle failed: {error}")
            }
            Self::Backend(error) => {
                write!(formatter, "Rendering backend failed: {error}")
            }
        }
    }
}

impl<E: Error + 'static> Error for CompositorRenderError<E> {}

impl AureaCompositor {
    /// Builds a snapshot of the currently mapped surfaces.
    pub fn render_scene(&self) -> RenderScene<'_> {
        RenderScene::from_window_manager(&self.window_manager)
    }

    /// Renders a previously requested frame on an enabled output.
    ///
    /// A frame request must be queued before calling this method.
    /// The frame lifecycle is completed even when the backend returns an error.
    ///
    /// The backend owns graphics-API details and is responsible for
    /// presenting the completed frame.
    pub fn render_output<B: RenderBackend>(
        &mut self,
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

        let token = self
            .begin_output_frame(output_id)
            .ok_or(CompositorRenderError::FrameNotRequested(output_id))?;

        let render_result = {
            let scene = self.render_scene();
            backend.render_frame(&target, &scene)
        };

        self.complete_output_frame(token)
            .map_err(CompositorRenderError::FrameLifecycle)?;

        render_result.map_err(CompositorRenderError::Backend)
    }
}
