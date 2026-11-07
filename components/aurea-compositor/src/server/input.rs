// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! libinput backend registration and keyboard event routing.

use std::io;

use smithay::{
    backend::{
        input::{Event, InputEvent, KeyboardKeyEvent},
        libinput::{LibinputInputBackend, LibinputSessionInterface},
    },
    reexports::{calloop::LoopHandle, input::Libinput},
};

use crate::drm::session::DrmSession;

use super::state::WaylandState;

/// Registers the seat's libinput source with Calloop.
pub(super) fn register_input_backend(
    loop_handle: &LoopHandle<'_, WaylandState>,
    session: &DrmSession,
) -> io::Result<()> {
    let interface = LibinputSessionInterface::from(session.session_handle());
    let mut context = Libinput::new_with_udev(interface);
    context
        .udev_assign_seat(&session.seat())
        .map_err(|_| io::Error::other("Failed to assign libinput to the active seat"))?;

    let backend = LibinputInputBackend::new(context);
    loop_handle
        .insert_source(backend, |event, _, state| {
            dispatch_input_event(event, state)
        })
        .map_err(|error| io::Error::other(error.to_string()))?;
    Ok(())
}

fn dispatch_input_event(event: InputEvent<LibinputInputBackend>, state: &mut WaylandState) {
    if let InputEvent::Keyboard { event } = event {
        state.compositor.handle_keyboard_input(
            event.key_code(),
            event.state(),
            event.time() as u32,
        );
    }
}
