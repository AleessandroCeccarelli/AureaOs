// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Dispatches libinput events to the compositor's input subsystems.

use std::sync::atomic::{AtomicU64, Ordering};

use smithay::backend::{
    input::{
        AbsolutePositionEvent, Axis, Event, GestureBeginEvent, GestureEndEvent,
        GesturePinchUpdateEvent as BackendPinchUpdateEvent,
        GestureSwipeUpdateEvent as BackendSwipeUpdateEvent, InputEvent, KeyboardKeyEvent,
        PointerAxisEvent, PointerButtonEvent, PointerMotionEvent, SwitchToggleEvent, TouchEvent,
    },
    libinput::LibinputInputBackend,
};

use aurea_desktop_api::input::{InputDeviceId, InputDeviceProvider};

use crate::input::pointing;

use super::{refresh_touchpad_settings, wayland_time};
use super::super::state::WaylandState;

static POINTER_MOTION_EVENTS: AtomicU64 = AtomicU64::new(0);

fn is_registered_pointing_device(state: &WaylandState, device: InputDeviceId) -> bool {
    state.compositor.pointing_devices().device(device).is_some()
}

pub(super) fn dispatch_input_event(event: InputEvent<LibinputInputBackend>, state: &mut WaylandState) {
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
                device.name(),
                device.sysname(),
                device_id.0,
                device.has_capability(smithay::reexports::input::DeviceCapability::Pointer),
                device.has_capability(smithay::reexports::input::DeviceCapability::Touch),
                device.has_capability(smithay::reexports::input::DeviceCapability::Gesture),
                device.has_capability(smithay::reexports::input::DeviceCapability::TabletTool),
                device.has_capability(smithay::reexports::input::DeviceCapability::TabletPad),
            );
            eprintln!(
                "AureaOS input: touchpad indicators tap_fingers={} disable_while_typing_available={}",
                device.config_tap_finger_count(),
                device.config_dwt_is_available()
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
                    .register_pointing_device(descriptor.clone());
                if descriptor.kind == aurea_desktop_api::input::InputDeviceKind::Touchpad {
                    state.touchpad_devices.insert(device_id, device.clone());
                } else {
                    // DeviceAdded may refresh an existing sysname with a new
                    // classification. Drop touchpad-only runtime state then.
                    state.touchpad_devices.remove(&device_id);
                    state.applied_touchpad_settings.remove(device_id);
                }
                if let Some(report) = touchpad_report {
                    state
                        .compositor
                        .record_touchpad_configuration(device_id, report);
                }
                state.compositor.sync_touchscreen_capability();
            } else if state.compositor.pointing_devices().device(device_id).is_some() {
                // A sysname can be re-announced with capabilities that no
                // longer describe a supported pointing device. Retire the
                // previous registration instead of leaving stale seat state.
                tracing::info!(
                    id = device_id.0,
                    "Re-announced device is no longer a supported pointing device"
                );
                state.compositor.disconnect_pointing_device(device_id);
                state.applied_touchpad_settings.remove(device_id);
                state.touchpad_devices.remove(&device_id);
                state.compositor.sync_touchscreen_capability();
            }
        }
        InputEvent::DeviceRemoved { device } => {
            eprintln!(
                "AureaOS input: device removed name={:?} sysname={:?}",
                device.name(),
                device.sysname()
            );
            if device.has_capability(smithay::reexports::input::DeviceCapability::TabletTool)
                || device.has_capability(smithay::reexports::input::DeviceCapability::TabletPad)
            {
                // A pad-only device still belongs to the tablet seat lifecycle,
                // even though Smithay 0.7 does not expose pad input events.
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

            // Cancel active protocol/input state before dropping the device
            // descriptor. This keeps cleanup independent of inventory lookups
            // and ensures held buttons/contacts are released while the seat is live.
            state.compositor.disconnect_pointing_device(id);
            state.applied_touchpad_settings.remove(id);
            state.touchpad_devices.remove(&id);
            state.compositor.sync_touchscreen_capability();
        }
        InputEvent::Keyboard { event } => {
            state.compositor.handle_keyboard_input(
                event.key_code(),
                event.state(),
                wayland_time(event.time()),
            );
        }
        InputEvent::PointerMotionAbsolute { event } => {
            if !is_registered_pointing_device(state, pointing::id_for(&event.device())) {
                return;
            }
            // The compositor currently routes absolute pointer devices only
            // when a single enabled output makes the target unambiguous.
            state
                .compositor
                .handle_pointer_motion_absolute(&event, wayland_time(event.time()));
        }
        InputEvent::PointerMotion { event } => {
            let delta = event.delta();
            let count = POINTER_MOTION_EVENTS.fetch_add(1, Ordering::Relaxed) + 1;
            if count == 1 || count % 120 == 0 {
                let device = event.device();
                eprintln!(
                    "AureaOS input: pointer motion count={} device={:?} sysname={:?} delta=({:.3},{:.3})",
                    count,
                    device.name(),
                    device.sysname(),
                    delta.x,
                    delta.y
                );
            }
            let unaccelerated_delta = event.delta_unaccel();

            let mut device = event.device();
            if !is_registered_pointing_device(state, pointing::id_for(&device)) {
                return;
            }
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
            let device_name = event.device().name().to_owned();
            eprintln!(
                "AureaOS input: pointer button device={:?} code={} state={:?}",
                device_name,
                event.button_code(),
                event.state()
            );
            let mut device = event.device();
            let device_id = pointing::id_for(&device);
            if !is_registered_pointing_device(state, device_id) {
                return;
            }
            refresh_touchpad_settings(state, &mut device);
            state.compositor.handle_pointer_button(
                device_id,
                event.button_code(),
                event.state(),
                wayland_time(event.time()),
            );
        }
        InputEvent::PointerAxis { event } => {
            let device_name = event.device().name().to_owned();
            eprintln!(
                "AureaOS input: pointer axis device={:?} horizontal={:?} vertical={:?} source={:?}",
                device_name,
                event.amount(Axis::Horizontal),
                event.amount(Axis::Vertical),
                event.source()
            );
            let mut device = event.device();
            if !is_registered_pointing_device(state, pointing::id_for(&device)) {
                return;
            }
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
            if !is_registered_pointing_device(state, pointing::id_for(&input_device)) {
                return;
            }
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
            if !is_registered_pointing_device(state, pointing::id_for(&input_device)) {
                return;
            }
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
            if !is_registered_pointing_device(state, pointing::id_for(&input_device)) {
                return;
            }
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
            if !is_registered_pointing_device(state, pointing::id_for(&input_device)) {
                return;
            }
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
            if !is_registered_pointing_device(state, pointing::id_for(&input_device)) {
                return;
            }
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
            if !is_registered_pointing_device(state, pointing::id_for(&input_device)) {
                return;
            }
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
            if !is_registered_pointing_device(state, pointing::id_for(&input_device)) {
                return;
            }
            refresh_touchpad_settings(state, &mut input_device);
            let device = pointing::id_for(&input_device);
            state
                .compositor
                .handle_hold_begin(device, event.fingers(), wayland_time(event.time()));
        }
        InputEvent::GestureHoldEnd { event } => {
            let mut input_device = event.device();
            if !is_registered_pointing_device(state, pointing::id_for(&input_device)) {
                return;
            }
            refresh_touchpad_settings(state, &mut input_device);
            let device = pointing::id_for(&input_device);
            state
                .compositor
                .handle_hold_end(device, event.cancelled(), wayland_time(event.time()));
        }
        InputEvent::TouchDown { event } => {
            let device = pointing::id_for(&event.device());
            if !is_registered_pointing_device(state, device) {
                return;
            }
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
            if !is_registered_pointing_device(state, device) {
                return;
            }
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
            if !is_registered_pointing_device(state, device) {
                return;
            }
            state
                .compositor
                .handle_touch_up(device, event.slot(), wayland_time(event.time()));
        }
        InputEvent::TouchCancel { event } => {
            let device = pointing::id_for(&event.device());
            if !is_registered_pointing_device(state, device) {
                return;
            }
            state
                .compositor
                .handle_touch_cancel(device, wayland_time(event.time()));
        }
        InputEvent::TouchFrame { .. } => {
            state.compositor.handle_touch_frame();
        }
        InputEvent::SwitchToggle { event } => {
            // Lid and tablet-mode switches are reported by libinput, but their
            // system policy is owned by the session/power-management layer.
            tracing::info!(
                device = ?event.device().name(),
                switch = ?event.switch(),
                state = ?event.state(),
                "Input switch state changed"
            );
        }
        InputEvent::TabletToolProximity { event } => {
            // Tablet-tool events may already be queued when DeviceRemoved is
            // dispatched. Never let a stale event recreate seat tablet/tool state.
            if !is_registered_pointing_device(state, pointing::id_for(&event.device())) {
                return;
            }
            state.compositor.handle_tablet_proximity(event);
        }
        InputEvent::TabletToolAxis { event } => {
            if !is_registered_pointing_device(state, pointing::id_for(&event.device())) {
                return;
            }
            state.compositor.handle_tablet_axis(event);
        }
        InputEvent::TabletToolTip { event } => {
            if !is_registered_pointing_device(state, pointing::id_for(&event.device())) {
                return;
            }
            state.compositor.handle_tablet_tip(event);
        }
        InputEvent::TabletToolButton { event } => {
            if !is_registered_pointing_device(state, pointing::id_for(&event.device())) {
                return;
            }
            state.compositor.handle_tablet_button(event);
        }
        _ => {}
    }
}

