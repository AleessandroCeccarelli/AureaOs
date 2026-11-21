// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Wayland seat and pointer-gesture protocol delegation.

use crate::AureaCompositor;
use smithay::{delegate_pointer_gestures, delegate_seat, delegate_tablet_manager};

delegate_seat!(AureaCompositor);
delegate_pointer_gestures!(AureaCompositor);
delegate_tablet_manager!(AureaCompositor);
