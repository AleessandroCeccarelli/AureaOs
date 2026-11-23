// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Compatibility name for the shared AureaOS active-display policy.

/// Maximum active output count, sourced from the public desktop API.
pub use aurea_desktop_api::display::MAX_ACTIVE_DISPLAYS as MAX_ACTIVE_OUTPUTS;
