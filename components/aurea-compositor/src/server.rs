// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Wayland server modules and public API.

mod builder;
mod client;
mod display_events;
mod event_loop;

#[cfg(feature = "drm")]
mod gbm_lifecycle;

#[cfg(feature = "drm")]
mod input;

mod output;
mod seat;
mod session;
mod session_events;
mod socket;
mod state;

#[cfg(feature = "drm")]
mod udev_events;

#[cfg(feature = "drm")]
mod rendering;

pub use event_loop::WaylandServer;
