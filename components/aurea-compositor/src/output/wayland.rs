// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Wayland protocol representation of display outputs.

use std::{error::Error, fmt};

use smithay::{
    output::{Mode, Output, PhysicalProperties, Scale, Subpixel},
    reexports::wayland_server::DisplayHandle,
    utils::{Point, Transform},
};

use crate::output::{
    layout::OutputPosition,
    manager::OutputManager,
    OutputId, OutputMode,
};

/// Error returned when an output cannot be represented by Wayland.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputPublicationError {
    output: OutputId,
    field: &'static str,
    value: u32,
}

impl OutputPublicationError {
    fn value_out_of_range(
        output: OutputId,
        field: &'static str,
        value: u32,
    ) -> Self {
        Self {
            output,
            field,
            value,
        }
    }
}

impl fmt::Display for OutputPublicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Output {} has a {} value ({}) that exceeds the Wayland range",
            self.output,
            self.field,
            self.value,
        )
    }
}

impl Error for OutputPublicationError {}

/// Validated Wayland representation of a display output.
struct WaylandOutputConfig {
    name: String,
    modes: Vec<Mode>,
    current_mode: Mode,
    scale: Scale,
    position: OutputPosition,
}

impl WaylandOutputConfig {
    /// Converts an enabled display output into validated Wayland data.
    fn from_output(
        output: &crate::output::DisplayOutput,
        position: OutputPosition,
    ) -> Result<Self, OutputPublicationError> {
        let modes = output
            .modes()
            .iter()
            .map(|mode| Self::convert_mode(output.id(), *mode))
            .collect::<Result<Vec<_>, _>>()?;

        let current_mode =
            Self::convert_mode(output.id(), output.current_mode())?;

        let scale = i32::try_from(output.scale()).map_err(|_| {
            OutputPublicationError::value_out_of_range(
                output.id(),
                "scale",
                output.scale(),
            )
        })?;

        Ok(Self {
            name: output.name().to_owned(),
            modes,
            current_mode,
            scale: Scale::Integer(scale),
            position,
        })
    }

    /// Converts a display mode into its Wayland representation.
    fn convert_mode(
        output: OutputId,
        mode: OutputMode,
    ) -> Result<Mode, OutputPublicationError> {
        let width = i32::try_from(mode.width()).map_err(|_| {
            OutputPublicationError::value_out_of_range(
                output,
                "mode width",
                mode.width(),
            )
        })?;

        let height = i32::try_from(mode.height()).map_err(|_| {
            OutputPublicationError::value_out_of_range(
                output,
                "mode height",
                mode.height(),
            )
        })?;

        let refresh = i32::try_from(mode.refresh_millihz()).map_err(|_| {
            OutputPublicationError::value_out_of_range(
                output,
                "refresh rate",
                mode.refresh_millihz(),
            )
        })?;

        Ok(Mode {
            size: (width, height).into(),
            refresh,
        })
    }
}

/// Creates Smithay outputs and publishes their `wl_output` globals.
///
/// All enabled outputs are validated before any Wayland global is created.
/// The returned outputs must be retained by the compositor for their
/// lifetime; dropping them removes the corresponding globals.
pub fn publish_outputs(
    display: &DisplayHandle,
    output_manager: &OutputManager,
) -> Result<Vec<Output>, OutputPublicationError> {
    let configurations = output_manager
        .registry()
        .iter()
        .filter(|output| output.is_enabled())
        .map(|output| {
            let position = output_manager
                .layout()
                .get(output.id())
                .unwrap_or_default();

            WaylandOutputConfig::from_output(output, position)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let outputs = configurations
        .into_iter()
        .map(|config| {
            let properties = PhysicalProperties {
                size: (0, 0).into(),
                subpixel: Subpixel::Unknown,
                make: "AureaOS".to_owned(),
                model: config.name.clone(),
                serial_number: String::new(),
            };

            let wayland_output = Output::new(config.name, properties);

            for mode in config.modes {
                wayland_output.add_mode(mode);
            }

            wayland_output.set_preferred(config.current_mode);

            wayland_output.change_current_state(
                Some(config.current_mode),
                Some(Transform::Normal),
                Some(config.scale),
                Some(Point::from((
                    config.position.x(),
                    config.position.y(),
                ))),
            );

            wayland_output.create_global::<crate::compositor::AureaCompositor>(
                display,
            );

            wayland_output
        })
        .collect();

    Ok(outputs)
}
