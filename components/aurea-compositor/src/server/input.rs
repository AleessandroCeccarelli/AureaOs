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
            AbsolutePositionEvent, KeyboardKeyEvent, PointerAxisEvent, PointerButtonEvent,
            PointerMotionEvent, TouchEvent,
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
        InputEvent::DeviceAdded { mut device } => {
            if let Some(report) = pointing::configure_touchpad(&mut device) {
                tracing::info!(?report, "Applied supported touchpad defaults");
            }

            if let Some(descriptor) = pointing::describe(&device) {
                tracing::info!(
                    device = %descriptor.name,
                    kind = ?descriptor.kind,
                    id = descriptor.id.0,
                    "Pointing device connected"
                );
                state.compositor.pointing_devices().connect(descriptor);
                state.compositor.sync_touchscreen_capability();
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
                state.compositor.clear_pointing_device_state(descriptor.id);
                state.compositor.sync_touchscreen_capability();
            }
        }
        InputEvent::Keyboard { event } => {
            state.compositor.handle_keyboard_input(
                event.key_code(),
                event.state(),
                event.time().millis(),
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
                event.time().millis(),
            );
        }
        InputEvent::PointerButton { event } => {
            state.compositor.handle_pointer_button(
                event.button_code(),
                event.state(),
                event.time().millis(),
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
                event.time().millis(),
            );
        }
        InputEvent::GestureSwipeBegin { event } => {
            let device = pointing::id_for(&event.device());
            state.compositor.handle_swipe_begin(device, event.fingers(), event.time());
        }
        InputEvent::GestureSwipeUpdate { event } => {
            let device = pointing::id_for(&event.device());
            state.compositor.handle_swipe_update(
                device,
                event.delta_x(),
                event.delta_y(),
                event.time().millis(),
            );
        }
        InputEvent::GestureSwipeEnd { event } => {
            let device = pointing::id_for(&event.device());
            state.compositor.handle_swipe_end(device, event.cancelled(), event.time());
        }
        InputEvent::GesturePinchBegin { event } => {
            let device = pointing::id_for(&event.device());
            state.compositor.handle_pinch_begin(device, event.fingers(), event.time());
        }
        InputEvent::GesturePinchUpdate { event } => {
            let device = pointing::id_for(&event.device());
            state.compositor.handle_pinch_update(
                device,
                event.delta_x(),
                event.delta_y(),
                event.scale(),
                event.rotation(),
                event.time(),
            );
        }
        InputEvent::GesturePinchEnd { event } => {
            let device = pointing::id_for(&event.device());
            state.compositor.handle_pinch_end(device, event.cancelled(), event.time());
        }
        InputEvent::GestureHoldBegin { event } => {
            let device = pointing::id_for(&event.device());
            state.compositor.handle_hold_begin(device, event.fingers(), event.time());
        }
        InputEvent::GestureHoldEnd { event } => {
            let device = pointing::id_for(&event.device());
            state.compositor.handle_hold_end(device, event.cancelled(), event.time());
        }
        InputEvent::TouchDown { event } => {
            let device = pointing::id_for(&event.device());
            if let Some((width, height)) = state.compositor.touchscreen_extent(device) {
                state.compositor.handle_touch_down(
                    device,
                    event.slot(),
                    event.x_transformed(width),
                    event.y_transformed(height),
                    event.time().millis(),
                );
            }
        }
        InputEvent::TouchMotion { event } => {
            let device = pointing::id_for(&event.device());
            if let Some((width, height)) = state.compositor.touchscreen_extent(device) {
                state.compositor.handle_touch_motion(
                    device,
                    event.slot(),
                    event.x_transformed(width),
                    event.y_transformed(height),
                    event.time().millis(),
                );
            }
        }
        InputEvent::TouchUp { event } => {
            let device = pointing::id_for(&event.device());
            state.compositor.handle_touch_up(
                device,
                event.slot(),
                event.time().millis(),
            );
        }
        InputEvent::TouchCancel { event } => {
            let device = pointing::id_for(&event.device());
            state.compositor.handle_touch_cancel(device, event.time().millis());
        }
        InputEvent::TouchFrame { .. } => {
            state.compositor.handle_touch_frame();
        }
        _ => {}
    }
}
