// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! libinput backend registration and input event routing.

use std::io;

use smithay::{
    backend::{
        input::{
            Axis, Event, GestureBeginEvent, GestureEndEvent,
            GesturePinchUpdateEvent as BackendPinchUpdateEvent,
            GestureSwipeUpdateEvent as BackendSwipeUpdateEvent, InputEvent,
            KeyboardKeyEvent, PointerAxisEvent, PointerButtonEvent, PointerMotionEvent,
        },
        libinput::{LibinputInputBackend, LibinputSessionInterface},
    },
    reexports::{calloop::LoopHandle, input::Libinput},
};

use crate::{drm::session::DrmSession, input::pointing};

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

    eprintln!(
        "AureaOS input: libinput assigned to seat {}",
        session.seat()
    );

    let backend = LibinputInputBackend::new(context);
    loop_handle
        .insert_source(backend, |event, _, state| {
            dispatch_input_event(event, state)
        })
        .map_err(|error| io::Error::other(error.to_string()))?;
    Ok(())
}

fn dispatch_input_event(event: InputEvent<LibinputInputBackend>, state: &mut WaylandState) {
    // Do not forward stale or queued input while libseat has paused this
    // compositor session (for example, after switching to another VT).
    if !state.session_active {
        return;
    }

    match event {
        InputEvent::DeviceAdded { device } => {
            if let Some(descriptor) = pointing::describe(&device) {
                tracing::info!(
                    device = %descriptor.name,
                    kind = ?descriptor.kind,
                    id = descriptor.id.0,
                    "Pointing device connected"
                );
                state.compositor.pointing_devices().connect(descriptor);
            }
        }
        InputEvent::DeviceRemoved { device } => {
            if let Some(descriptor) = pointing::describe(&device) {
                tracing::info!(
                    device = %descriptor.name,
                    id = descriptor.id.0,
                    "Pointing device disconnected"
                );
                state.compositor.pointing_devices().disconnect(descriptor.id);
            }
        }
        InputEvent::Keyboard { event } => {
            state.compositor.handle_keyboard_input(
                event.key_code(),
                event.state(),
                event.time() as u32,
            );
        }
        InputEvent::PointerMotion { event } => {
            let delta = event.delta();
            let unaccelerated_delta = event.delta_unaccel();

            let mut device = event.device();
            crate::input::pointer_acceleration::apply(
                &mut device,
                state.compositor.pointer_settings(),
            );
            state.compositor.handle_pointer_motion(
                delta,
                unaccelerated_delta,
                event.time() as u32,
            );
        }
        InputEvent::PointerButton { event } => {
            state.compositor.handle_pointer_button(
                event.button_code(),
                event.state(),
                event.time() as u32,
            );
        }
        InputEvent::PointerAxis { event } => {
            state.compositor.handle_pointer_axis(
                event.amount(Axis::Horizontal),
                event.amount(Axis::Vertical),
                event.amount_v120(Axis::Horizontal),
                event.amount_v120(Axis::Vertical),
                event.source(),
                event.relative_direction(Axis::Horizontal),
                event.relative_direction(Axis::Vertical),
                event.time() as u32,
            );
        }
        InputEvent::GestureSwipeBegin { event } => {
            state.compositor.handle_swipe_begin(event.fingers(), event.time());
        }
        InputEvent::GestureSwipeUpdate { event } => {
            state.compositor.handle_swipe_update(
                event.delta_x(),
                event.delta_y(),
                event.time().millis(),
            );
        }
        InputEvent::GestureSwipeEnd { event } => {
            state.compositor.handle_swipe_end(event.cancelled(), event.time());
        }
        InputEvent::GesturePinchBegin { event } => {
            state.compositor.handle_pinch_begin(event.fingers(), event.time());
        }
        InputEvent::GesturePinchUpdate { event } => {
            state.compositor.handle_pinch_update(
                event.delta_x(),
                event.delta_y(),
                event.scale(),
                event.rotation(),
                event.time(),
            );
        }
        InputEvent::GesturePinchEnd { event } => {
            state.compositor.handle_pinch_end(event.cancelled(), event.time());
        }
        InputEvent::GestureHoldBegin { event } => {
            state.compositor.handle_hold_begin(event.fingers(), event.time());
        }
        InputEvent::GestureHoldEnd { event } => {
            state.compositor.handle_hold_end(event.cancelled(), event.time());
        }
        _ => {}
    }
}
