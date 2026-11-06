
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Native session event registration and handling.

#[cfg(feature = "drm")]
use std::io;

#[cfg(feature = "drm")]
use smithay::{
    backend::session::{Event as SessionEvent, Session},
    reexports::calloop::LoopHandle,
};

#[cfg(feature = "drm")]
use crate::drm::{device::DrmDevice, DrmDeviceError, DrmSession};

#[cfg(feature = "drm")]
use super::state::WaylandState;

/// Applies a session event to the compositor and DRM device state.
#[cfg(feature = "drm")]
fn apply_session_event(
    event: SessionEvent,
    session_active: &mut bool,
    drm_device: Option<&mut DrmDevice>,
) -> Result<(), DrmDeviceError> {
    match event {
        SessionEvent::PauseSession => {
            if let Some(device) = drm_device {
                device.pause();
            }

            *session_active = false;
        }
        SessionEvent::ActivateSession => {
            if let Some(device) = drm_device {
                device.activate()?;
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
            let result = apply_session_event(
                event,
                &mut state.session_active,
                state._drm_device.as_mut(),
            );

            if let Err(error) = result {
                state.error = Some(io::Error::other(error));
            }
        })
        .map_err(io::Error::other)?;

    Ok(())
}

#[cfg(all(test, feature = "drm"))]
mod tests {
    use super::*;

    #[test]
    fn pause_event_updates_session_state() {
        let mut active = true;

        let result =
            apply_session_event(SessionEvent::PauseSession, &mut active, None);

        assert!(result.is_ok());
        assert!(!active);
    }

    #[test]
    fn activate_event_updates_session_state() {
        let mut active = false;

        let result =
            apply_session_event(SessionEvent::ActivateSession, &mut active, None);

        assert!(result.is_ok());
        assert!(active);
    }
}
