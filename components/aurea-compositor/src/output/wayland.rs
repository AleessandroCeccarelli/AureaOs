
 // SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Wayland protocol representation of display outputs.

use smithay::{
    output::{Mode, Output, PhysicalProperties, Scale, Subpixel},
    reexports::wayland_server::DisplayHandle,
    utils::{Point, Transform},
};

use crate::output::manager::OutputManager;

/// Creates Smithay outputs and publishes their `wl_output` globals.
///
/// The returned outputs must be retained by the compositor for their
/// lifetime; dropping them would remove the corresponding globals.
pub fn publish_outputs(
    display: &DisplayHandle,
    output_manager: &OutputManager,
) -> Vec<Output> {
    output_manager
        .registry()
        .iter()
        .filter(|output| output.is_enabled())
        .map(|output| {
            let properties = PhysicalProperties {
                size: (0, 0).into(),
                subpixel: Subpixel::Unknown,
                make: "AureaOS".to_owned(),
                model: output.name().to_owned(),
                serial_number: String::new(),
            };

            let wayland_output = Output::new(output.name().to_owned(), properties);

            for mode in output.modes() {
                wayland_output.add_mode(Mode {
                    size: (
                        i32::try_from(mode.width()).unwrap_or(i32::MAX),
                        i32::try_from(mode.height()).unwrap_or(i32::MAX),
                    )
                        .into(),
                    refresh: i32::try_from(mode.refresh_millihz())
                        .unwrap_or(i32::MAX),
                });
            }

            let current_mode = output.current_mode();
            let current_mode = Mode {
                size: (
                    i32::try_from(current_mode.width()).unwrap_or(i32::MAX),
                    i32::try_from(current_mode.height()).unwrap_or(i32::MAX),
                )
                    .into(),
                refresh: i32::try_from(current_mode.refresh_millihz())
                    .unwrap_or(i32::MAX),
            };

            wayland_output.set_preferred(current_mode);

            let position = output_manager
                .layout()
                .get(output.id())
                .unwrap_or_default();

            wayland_output.change_current_state(
                Some(current_mode),
                Some(Transform::Normal),
                Some(Scale::Integer(
                    i32::try_from(output.scale()).unwrap_or(i32::MAX),
                )),
                Some(Point::from((position.x(), position.y()))),
            );

            wayland_output.create_global::<crate::compositor::AureaCompositor>(display);
            wayland_output
        })
        .collect()
}
