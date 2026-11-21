// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Hardware-independent interaction policy shared by pointing-device providers.

mod engine;
mod gesture;
mod profile;

pub use engine::InteractionEngine;
pub use gesture::{GesturePhase, PointingGesture, PointingInteraction};
pub use profile::{GestureBinding, GestureProfile};
