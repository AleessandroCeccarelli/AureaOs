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
            // Attempt all devices so one failure does not leave later devices
            // stale. The first error is reported and the session remains
            // inactive until every managed device can be reactivated.
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

/// Registers libseat activation and pause events with the event loop.
#[cfg(feature = "drm")]
pub(super) fn register_session_events(
    loop_handle: &LoopHandle<'_, WaylandState>,
    session: &mut DrmSession,
) -> io::Result<()> {
    let notifier = session.take_notifier().map_err(io::Error::other)?;

    loop_handle
        .insert_source(notifier, |event, _, state| {
            let result =
                apply_session_event(event, &mut state.session_active, &mut state.drm_devices);

            if let Err(error) = result {
                state.error = Some(io::Error::other(error));
            }
        })
        .map_err(|_| io::Error::other("Failed to register the libseat session event notifier"))?;

    Ok(())
}

#[cfg(all(test, feature = "drm"))]
#[path = "../../tests/unit/server_session_events.rs"]
mod tests;
