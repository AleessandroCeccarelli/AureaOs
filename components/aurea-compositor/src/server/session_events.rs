
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
use crate::drm::DrmSession;

#[cfg(feature = "drm")]
use super::state::WaylandState;

/// Applies a libseat event to the compositor's session state.
#[cfg(feature = "drm")]
fn apply_session_event(
    event: SessionEvent,
    session_active: &mut bool,
) {
    match event {
        SessionEvent::ActivateSession => *session_active = true,
        SessionEvent::PauseSession => *session_active = false,
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
            apply_session_event(event, &mut state.session_active);
        })
        .map_err(io::Error::other)?;

    Ok(())
}

#[cfg(all(test, feature = "drm"))]
mod tests {
    use super::*;

    #[test]
    fn session_events_update_active_state() {
        let mut active = false;

        apply_session_event(SessionEvent::ActivateSession, &mut active);
        assert!(active);

        apply_session_event(SessionEvent::PauseSession, &mut active);
        assert!(!active);
    }
}
