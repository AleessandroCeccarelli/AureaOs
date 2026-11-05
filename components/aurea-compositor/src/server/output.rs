
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Wayland output protocol handling.

use smithay::{
    delegate_output,
    wayland::output::OutputHandler,
};

use crate::AureaCompositor;

/// Handles Wayland output protocol requests.
impl OutputHandler for AureaCompositor {}

delegate_output!(AureaCompositor);
