// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Wayland server modules and public API.

mod client;
mod display_events;
mod event_loop;
mod output;
mod session;
mod session_events;
mod socket;
mod state;

pub use event_loop::WaylandServer;
