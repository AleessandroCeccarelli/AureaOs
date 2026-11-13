// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Native session event registration and handling.

#[cfg(feature = "drm")]
use std::io;

#[cfg(feature = "drm")]
use smithay::{backend::session::Event as SessionEvent, reexports::calloop::LoopHandle};

#[cfg(feature = "drm")]
use crate::drm::{DrmSession, device::DrmDeviceError, device_manager::ManagedDrmDevice};

#[cfg(feature = "drm")]
use super::state::WaylandState;

/// Applies a session event to the compositor and DRM device state.
#[cfg(feature = "drm")]
fn apply_session_event<D, K>(
    event: SessionEvent,
    session_active: &mut bool,
    drm_devices: &mut crate::drm::device_manager::DrmDeviceManager<D, K>,
) -> Result<(), DrmDeviceError>
where
    D: ManagedDrmDevice<Identity = K>,
    K: Copy + Eq + std::hash::Hash + std::fmt::Display,
{
    match event {
        SessionEvent::PauseSession => {
            for (_, device) in drm_devices.iter_mut() {
                device.pause();
            }

            *session_active = false;
        }

        SessionEvent::ActivateSession => {
            let mut first_error = None;

            for (_, device) in drm_devices.iter_mut() {
                if let Err(error) = device.activate() {
                    first_error.get_or_insert(error);
                }
            }

            if let Some(error) = first_error {
                *session_active = false;
                return Err(error);
            }

            *session_active = true;
        }
    }

    Ok(())
}


/// Clears every managed KMS surface while the libseat session still owns DRM.
#[cfg(feature = "drm")]
pub(super) fn clear_drm_surfaces(state: &mut WaylandState) -> Option<String> {
    let mut errors = Vec::new();

    for (node, surfaces) in state.drm_surfaces.iter_mut() {
        if let Err(error) = surfaces.clear() {
            errors.push(format!("DRM device {node}: {error}"));
        }
    }

    if errors.is_empty() {
        None
    } else {
        Some(errors.join("; "))
    }
}

/// Registers libseat activation and pause events with the event loop.
#[cfg(feature = "drm")]
pub(super) fn register_session_events(
    loop_handle: &LoopHandle<'_, WaylandState>,
    session: &mut DrmSession,
) -> io::Result<()> {
    let notifier = session.take_notifier().map_err(io::Error::other)?;

    loop_handle
        .insert_source(notifier, |event, _, state| {
            let is_pause = matches!(event, SessionEvent::PauseSession);

            let cleanup_error = if is_pause {
                // Disable KMS planes while DRM access is still valid. Continue
                // releasing dependent resources even if one output fails.
                let error = clear_drm_surfaces(state);
                super::gbm_lifecycle::clear_gbm_devices(state);
                state.vulkan_backends.clear();
                error
            } else {
                None
            };

            let session_result =
                apply_session_event(event, &mut state.session_active, &mut state.drm_devices);

            if let Some(error) = cleanup_error {
                state.error = Some(io::Error::other(error));
                return;
            }

            if let Err(error) = session_result {
                state.error = Some(io::Error::other(error));
                return;
            }

            if state.session_active {
                super::udev_events::activate_discovered_devices(state);
            }
        })
        .map_err(|_| io::Error::other("Failed to register the libseat session event notifier"))?;

    Ok(())
}

#[cfg(all(test, feature = "drm"))]
#[path = "../../tests/unit/server_session_events.rs"]
mod tests;
