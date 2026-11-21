// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Coordinated pointing-device services.

mod discovery;
mod registry;

pub use registry::PointingDeviceService;
pub(super) use discovery::describe;
