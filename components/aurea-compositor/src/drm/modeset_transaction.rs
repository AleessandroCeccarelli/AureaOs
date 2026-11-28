// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Compatibility namespace for the DRM/KMS transaction subsystem.
//!
//! The implementation modules live directly under [`crate::drm`]. Keeping
//! this namespace as a re-export avoids declaring a second, nonexistent
//! `modeset_transaction/` source tree.

pub use super::*;
