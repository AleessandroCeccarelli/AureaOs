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
mod touchscreen_slots;
mod touchpad;
mod touchpad_settings;

pub use registry::PointingDeviceService;
pub use touchscreen::TouchscreenOutputMap;
pub use touchscreen_slots::TouchscreenSlotMap;
pub(super) use discovery::{describe, id_for};
