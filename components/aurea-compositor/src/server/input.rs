// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! libinput backend registration and input event routing.

use std::{io, sync::atomic::{AtomicU64, Ordering}};

use aurea_desktop_api::input::InputDeviceProvider;

use smithay::{
    backend::{
        input::{
            AbsolutePositionEvent, Axis, Event, GestureBeginEvent, GestureEndEvent,
            GesturePinchUpdateEvent as BackendPinchUpdateEvent,
            GestureSwipeUpdateEvent as BackendSwipeUpdateEvent, InputEvent, KeyboardKeyEvent,
            PointerAxisEvent, PointerButtonEvent, PointerMotionEvent, TouchEvent,
        },
        libinput::{LibinputInputBackend, LibinputSessionInterface},
    },
    reexports::{calloop::LoopHandle, input::Libinput},
};

use crate::{drm::session::DrmSession, input::pointing};

/// Converts libinput microsecond timestamps to wrapping Wayland millisecond timestamps.
fn wayland_time(microseconds: u64) -> u32 {
    (microseconds / 1_000) as u32
}

use super::state::WaylandState;

static POINTER_MOTION_EVENTS: AtomicU64 = AtomicU64::new(0);

/// Applies changed preferences without retaining libinput device handles.
fn refresh_touchpad_settings(
    state: &mut WaylandState,
    device: &mut smithay::reexports::input::Device,
) {
    let id = pointing::id_for(device);
    let settings = state.compositor.touchpad_settings_for(id);
    if !state.applied_touchpad_settings.needs_apply(id, settings) {
        return;
    }
    if let Some(report) = pointing::configure_touchpad(device, settings) {
        tracing::info!(device_id = id.0, ?report, "Updated touchpad preferences");
        state.compositor.record_touchpad_configuration(id, report);
        if !report.has_failures() {
            state.applied_touchpad_settings.mark_applied(id, settings);
        }
    }
}

/// Drains shell requests and applies them to retained live libinput devices.
pub(super) fn apply_pending_touchpad_settings(state: &mut WaylandState) {
    loop {
        let command = match state.touchpad_settings_receiver.try_recv() {
            Ok(command) => command,
            Err(std::sync::mpsc::TryRecvError::Empty)
            | Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
        };

        let Some(descriptor) = state.compositor.pointing_devices().device(command.device) else {
            tracing::debug!(
                device_id = command.device.0,
                "Ignoring touchpad settings for an unknown device"
            );
            continue;
        };
        if descriptor.kind != aurea_desktop_api::input::InputDeviceKind::Touchpad {
            tracing::debug!(
                device_id = command.device.0,
                "Ignoring touchpad settings for a non-touchpad device"
            );
            continue;
        }
        if !state
            .compositor
            .set_touchpad_device_settings(command.device, command.settings)
        {
            tracing::warn!(
                device_id = command.device.0,
                "Touchpad settings profile was rejected"
            );
            continue;
        }

        let Some(device) = state.touchpad_devices.get_mut(&command.device) else {
            tracing::warn!(
                device_id = command.device.0,
                "Touchpad has no live libinput handle"
            );
            continue;
        };
        let Some(report) = pointing::configure_touchpad(device, command.settings) else {
            tracing::warn!(
                device_id = command.device.0,
                "Live touchpad configuration is unavailable"
            );
            continue;
        };

        state
            .compositor
            .record_touchpad_configuration(command.device, report);
        if !report.has_failures() {
            state
                .applied_touchpad_settings
                .mark_applied(command.device, command.settings);
        }
        tracing::info!(
            device_id = command.device.0,
            ?report,
            "Applied live touchpad settings request"
        );
    }
}

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
        "AureaOS input: libinput context created and assigned to seat {}",
        session.seat()
    );

    let backend = LibinputInputBackend::new(context);
    loop_handle
        .insert_source(backend, |event, _, state| {
            dispatch_input_event(event, state)
        })
        .map_err(|error| io::Error::other(error.to_string()))?;
    eprintln!("AureaOS input: libinput backend registered in Calloop event loop");
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
            let device_id = pointing::id_for(&device);
            eprintln!(
                "AureaOS input: device added name={:?} sysname={:?} id={} pointer={} touch={} gesture={} tablet_tool={} tablet_pad={}",
                device.name(), device.sysname(), device_id.0,
                device.has_capability(smithay::reexports::input::DeviceCapability::Pointer),
                device.has_capability(smithay::reexports::input::DeviceCapability::Touch),
                device.has_capability(smithay::reexports::input::DeviceCapability::Gesture),
                device.has_capability(smithay::reexports::input::DeviceCapability::TabletTool),
                device.has_capability(smithay::reexports::input::DeviceCapability::TabletPad),
            );
            let touchpad_settings = state.compositor.touchpad_settings_for(device_id);
            let touchpad_report = pointing::configure_touchpad(&mut device, touchpad_settings);
            if let Some(report) = touchpad_report {
                tracing::info!(
                    device_id = device_id.0,
                    ?report,
                    "Applied supported touchpad defaults"
                );
                if !report.has_failures() {
                    state
                        .applied_touchpad_settings
                        .mark_applied(device_id, touchpad_settings);
                }
            }

            if let Some(descriptor) = pointing::describe(&device) {
                eprintln!(
                    "AureaOS input: pointing device classified kind={:?} name={:?} id={}",
                    descriptor.kind, descriptor.name, descriptor.id.0
                );
                tracing::info!(
                    device = %descriptor.name,
                    kind = ?descriptor.kind,
                    id = descriptor.id.0,
                    "Pointing device connected"
                );
                state
                    .compositor
                    .pointing_devices()
                    .connect(descriptor.clone());
                if descriptor.kind == aurea_desktop_api::input::InputDeviceKind::Touchpad {
                    state.touchpad_devices.insert(device_id, device.clone());
                }
                if let Some(report) = touchpad_report {
                    state
                        .compositor
                        .record_touchpad_configuration(device_id, report);
                }
                state.compositor.sync_touchscreen_capability();
            }
        }
        InputEvent::DeviceRemoved { device } => {
            eprintln!(
                "AureaOS input: device removed name={:?} sysname={:?}",
                device.name(), device.sysname()
            );
            if device.has_capability(smithay::reexports::input::DeviceCapability::TabletTool) {
                state.compositor.remove_tablet_device(&device);
            }
            // Removal must not depend on rediscovering capabilities: libinput
            // may already have withdrawn some metadata by the time this event
            // is delivered. The sysname-derived ID is sufficient to clean up
            // every registry and compositor-owned entry deterministically.
            let id = pointing::id_for(&device);
            let descriptor = state.compositor.pointing_devices().device(id);

            if let Some(descriptor) = descriptor {
                tracing::info!(
                    device = %descriptor.name,
                    id = id.0,
                    "Pointing device disconnected"
                );
            } else {
                tracing::debug!(id = id.0, "Unregistered input device removed");
            }

            state.applied_touchpad_settings.remove(id);
            state.touchpad_devices.remove(&id);
            state.compositor.pointing_devices().disconnect(id);
            state.compositor.clear_pointing_device_state(id);
            state.compositor.sync_touchscreen_capability();
        }
        InputEvent::Keyboard { event } => {
            state.compositor.handle_keyboard_input(
                event.key_code(),
                event.state(),
                wayland_time(event.time()),
            );
        }
        InputEvent::PointerMotion { event } => {
            let delta = event.delta();
            let count = POINTER_MOTION_EVENTS.fetch_add(1, Ordering::Relaxed) + 1;
            if count == 1 || count % 120 == 0 {
                let device = event.device();
                eprintln!(
                    "AureaOS input: pointer motion count={} device={:?} sysname={:?} delta=({:.3},{:.3})",
                    count, device.name(), device.sysname(), delta.x, delta.y
                );
            }
            let unaccelerated_delta = event.delta_unaccel();

            let mut device = event.device();
            refresh_touchpad_settings(state, &mut device);
            let settings = state
                .compositor
                .pointer_settings_for(pointing::id_for(&device));
            crate::input::pointer_acceleration::apply(&mut device, settings);
            state.compositor.handle_pointer_motion(
                delta,
                unaccelerated_delta,
                wayland_time(event.time()),
            );
        }
        InputEvent::PointerButton { event } => {
            let mut device = event.device();
            refresh_touchpad_settings(state, &mut device);
            state.compositor.handle_pointer_button(
                pointing::id_for(&event.device()),
                event.button_code(),
                event.state(),
                wayland_time(event.time()),
            );
        }
        InputEvent::PointerAxis { event } => {
            let mut device = event.device();
            refresh_touchpad_settings(state, &mut device);
            state.compositor.handle_pointer_axis(
                event.amount(Axis::Horizontal),
                event.amount(Axis::Vertical),
                event.amount_v120(Axis::Horizontal),
                event.amount_v120(Axis::Vertical),
                event.source(),
                event.relative_direction(Axis::Horizontal),
                event.relative_direction(Axis::Vertical),
                wayland_time(event.time()),
            );
        }
        InputEvent::GestureSwipeBegin { event } => {
            let mut input_device = event.device();
            refresh_touchpad_settings(state, &mut input_device);
            let device = pointing::id_for(&input_device);
            state.compositor.handle_swipe_begin(
                device,
                event.fingers(),
                wayland_time(event.time()),
            );
        }
        InputEvent::GestureSwipeUpdate { event } => {
            let mut input_device = event.device();
            refresh_touchpad_settings(state, &mut input_device);
            let device = pointing::id_for(&input_device);
            state.compositor.handle_swipe_update(
                device,
                event.delta_x(),
                event.delta_y(),
                wayland_time(event.time()),
            );
        }
        InputEvent::GestureSwipeEnd { event } => {
            let mut input_device = event.device();
            refresh_touchpad_settings(state, &mut input_device);
            let device = pointing::id_for(&input_device);
            state.compositor.handle_swipe_end(
                device,
                event.cancelled(),
                wayland_time(event.time()),
            );
        }
        InputEvent::GesturePinchBegin { event } => {
            let mut input_device = event.device();
            refresh_touchpad_settings(state, &mut input_device);
            let device = pointing::id_for(&input_device);
            state.compositor.handle_pinch_begin(
                device,
                event.fingers(),
                wayland_time(event.time()),
            );
        }
        InputEvent::GesturePinchUpdate { event } => {
            let mut input_device = event.device();
            refresh_touchpad_settings(state, &mut input_device);
            let device = pointing::id_for(&input_device);
            state.compositor.handle_pinch_update(
                device,
                event.delta_x(),
                event.delta_y(),
                event.scale(),
                event.rotation(),
                wayland_time(event.time()),
            );
        }
        InputEvent::GesturePinchEnd { event } => {
            let mut input_device = event.device();
            refresh_touchpad_settings(state, &mut input_device);
            let device = pointing::id_for(&input_device);
            state.compositor.handle_pinch_end(
                device,
                event.cancelled(),
                wayland_time(event.time()),
            );
        }
        InputEvent::GestureHoldBegin { event } => {
            let mut input_device = event.device();
            refresh_touchpad_settings(state, &mut input_device);
            let device = pointing::id_for(&input_device);
            state
                .compositor
                .handle_hold_begin(device, event.fingers(), wayland_time(event.time()));
        }
        InputEvent::GestureHoldEnd { event } => {
            let mut input_device = event.device();
            refresh_touchpad_settings(state, &mut input_device);
            let device = pointing::id_for(&input_device);
            state
                .compositor
                .handle_hold_end(device, event.cancelled(), wayland_time(event.time()));
        }
        InputEvent::TouchDown { event } => {
            let device = pointing::id_for(&event.device());
            if let Some((width, height)) = state.compositor.touchscreen_extent(device) {
                state.compositor.handle_touch_down(
                    device,
                    event.slot(),
                    event.x_transformed(width),
                    event.y_transformed(height),
                    wayland_time(event.time()),
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
                    wayland_time(event.time()),
                );
            }
        }
        InputEvent::TouchUp { event } => {
            let device = pointing::id_for(&event.device());
            state
                .compositor
                .handle_touch_up(device, event.slot(), wayland_time(event.time()));
        }
        InputEvent::TouchCancel { event } => {
            let device = pointing::id_for(&event.device());
            state
                .compositor
                .handle_touch_cancel(device, wayland_time(event.time()));
        }
        InputEvent::TouchFrame { .. } => {
            state.compositor.handle_touch_frame();
        }
        InputEvent::TabletToolProximity { event } => {
            state.compositor.handle_tablet_proximity(event);
        }
        InputEvent::TabletToolAxis { event } => {
            state.compositor.handle_tablet_axis(event);
        }
        InputEvent::TabletToolTip { event } => {
            state.compositor.handle_tablet_tip(event);
        }
        InputEvent::TabletToolButton { event } => {
            state.compositor.handle_tablet_button(event);
        }
        _ => {}
    }
}
