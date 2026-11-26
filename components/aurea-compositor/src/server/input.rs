// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! libinput backend registration and touchpad configuration lifecycle.

use std::io;

use aurea_desktop_api::input::InputDeviceProvider;

use smithay::{
    backend::libinput::{LibinputInputBackend, LibinputSessionInterface},
    reexports::{calloop::LoopHandle, input::Libinput},
};

use crate::{drm::session::DrmSession, input::pointing};

mod dispatch;

use super::state::WaylandState;

/// Converts libinput microsecond timestamps to wrapping Wayland millisecond timestamps.
pub(super) fn wayland_time(microseconds: u64) -> u32 {
    (microseconds / 1_000) as u32
}

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

/// Reapplies saved touchpad profiles after a libseat session becomes active.
///
/// The input backend can retain device handles across VT switches while the
/// underlying kernel devices and libinput configuration may have been reset.
/// Invalidate the cache first, then configure every retained touchpad handle.
pub(super) fn reapply_touchpad_settings(state: &mut WaylandState) {
    let devices = state.touchpad_devices.keys().copied().collect::<Vec<_>>();

    for id in devices {
        // A device-specific invalidation must never disturb another touchpad's
        // successfully applied settings or its retry state.
        state.applied_touchpad_settings.invalidate(id);
        let settings = state.compositor.touchpad_settings_for(id);
        let Some(device) = state.touchpad_devices.get_mut(&id) else {
            continue;
        };
        let Some(report) = pointing::configure_touchpad(device, settings) else {
            tracing::warn!(device_id = id.0, "Touchpad could not be reconfigured after session activation");
            continue;
        };

        state.compositor.record_touchpad_configuration(id, report);
        if !report.has_failures() {
            state.applied_touchpad_settings.mark_applied(id, settings);
        }
        tracing::info!(device_id = id.0, ?report, "Restored touchpad preferences after session activation");
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



/// Reapplies acceleration profiles after libseat reactivates the input session.
/// libinput may reset device configuration while a different VT owns the seat.
pub(super) fn reapply_pointer_acceleration(state: &mut WaylandState) {
    let mouse_ids = state.mouse_devices.keys().copied().collect::<Vec<_>>();
    for id in mouse_ids {
        let settings = state.compositor.pointer_settings_for(id);
        if let Some(device) = state.mouse_devices.get_mut(&id) {
            crate::input::pointer_acceleration::apply(device, settings);
            tracing::info!(device_id = id.0, "Restored mouse acceleration after session activation");
        }
    }

    // Touchpads share the acceleration adapter but retain their own handles
    // and profile ownership in the touchpad registry.
    let touchpad_ids = state.touchpad_devices.keys().copied().collect::<Vec<_>>();
    for id in touchpad_ids {
        let settings = state.compositor.pointer_settings_for(id);
        if let Some(device) = state.touchpad_devices.get_mut(&id) {
            pointing::apply_pointer_acceleration(device, settings);
        }
    }
}

/// Drains shell requests and applies them to the connected mouse immediately.
/// Mouse handles are retained only for the lifetime of their DeviceAdded event.
pub(super) fn apply_pending_pointer_settings(state: &mut WaylandState) {
    loop {
        let command = match state.pointer_settings_receiver.try_recv() {
            Ok(command) => command,
            Err(std::sync::mpsc::TryRecvError::Empty)
            | Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
        };

        let Some(descriptor) = state.compositor.pointing_devices().device(command.device) else {
            tracing::debug!(
                device_id = command.device.0,
                "Ignoring pointer settings for an unknown device"
            );
            continue;
        };
        if descriptor.kind != aurea_desktop_api::input::InputDeviceKind::Mouse {
            tracing::debug!(
                device_id = command.device.0,
                kind = ?descriptor.kind,
                "Ignoring mouse settings for a non-mouse device"
            );
            continue;
        }
        if !state.compositor.set_pointer_device_settings(command.device, command.settings) {
            tracing::warn!(
                device_id = command.device.0,
                "Pointer settings profile was rejected"
            );
            continue;
        }
        let Some(device) = state.mouse_devices.get_mut(&command.device) else {
            tracing::warn!(device_id = command.device.0, "Mouse has no live libinput handle");
            continue;
        };
        crate::input::pointer_acceleration::apply(device, command.settings);
        tracing::info!(
            device_id = command.device.0,
            profile = ?command.settings.profile(),
            speed = command.settings.speed(),
            "Applied live mouse preferences to libinput"
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
            dispatch::dispatch_input_event(event, state)
        })
        .map_err(|error| io::Error::other(error.to_string()))?;
    eprintln!("AureaOS input: libinput backend registered in Calloop event loop");
    Ok(())
}
