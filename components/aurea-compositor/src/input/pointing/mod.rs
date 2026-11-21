// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Coordinated pointing-device services.

mod discovery;
mod mouse;
mod registry;
mod tablet;
mod touchscreen;
mod touchpad;

pub use registry::PointingDeviceService;
pub use touchscreen::TouchscreenOutputMap;
pub(super) use discovery::{describe, id_for};
