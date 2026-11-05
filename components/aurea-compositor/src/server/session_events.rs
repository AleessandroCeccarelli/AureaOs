
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

/// Registers libseat activation and pause events with the event loop.
#[cfg(feature = "drm")]
pub(super) fn register_session_events(
    loop_handle: &LoopHandle<'_, WaylandState>,
    session: &mut DrmSession,
) -> io::Result<()> {
    let notifier = session.take_notifier().map_err(io::Error::other)?;

    loop_handle
        .insert_source(notifier, |event, _, state| {
            match event {
                SessionEvent::ActivateSession => {
                    state.session_active = true;
                }
                SessionEvent::PauseSession => {
                    state.session_active = false;
                }
            }
        })
        .map_err(io::Error::other)?;

    Ok(())
}
