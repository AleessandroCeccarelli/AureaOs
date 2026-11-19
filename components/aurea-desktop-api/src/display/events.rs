// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Observable display discovery and configuration lifecycle events.

use super::capabilities::{DisplayDescriptor, DisplayId};

/// Changes reported by the display capability/configuration service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayEvent {
    Connected(DisplayDescriptor),
    Disconnected(DisplayId),
    CapabilitiesChanged(DisplayDescriptor),
    ConfigurationChanged,
    PrimaryDisplayChanged(Option<DisplayId>),
}
