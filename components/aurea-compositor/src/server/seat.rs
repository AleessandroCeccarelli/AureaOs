// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Wayland seat protocol delegation.

use crate::AureaCompositor;
use smithay::delegate_seat;

delegate_seat!(AureaCompositor);
