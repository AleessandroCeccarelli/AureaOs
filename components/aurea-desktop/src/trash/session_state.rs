// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Reads the initial home Trash state for desktop session startup.

use std::{env, path::PathBuf};

use aurea_desktop_api::desktop::DesktopTrashState;

use super::TrashService;

/// Reads the current user's Trash state without transferring filesystem access
/// into the compositor. Live refresh is handled by a later session service.
pub(crate) fn initial_visual_state() -> Result<DesktopTrashState, String> {
    let home = env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "HOME is unset; the home Trash cannot be initialized".to_owned())?;

    let service = TrashService::new(PathBuf::from(home))
        .map_err(|error| format!("Cannot initialize the home Trash: {error}"))?;
    let snapshot = service
        .snapshot()
        .map_err(|error| format!("Cannot read the home Trash state: {error}"))?;

    Ok(snapshot.visual_state().into())
}
