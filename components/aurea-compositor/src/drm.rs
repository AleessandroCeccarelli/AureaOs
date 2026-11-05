
 // SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Native DRM/KMS backend modules.

pub mod device;
pub mod discovery;
pub mod output;
pub mod outputs;
pub mod session;

pub use session::{DrmSession, DrmSessionError};
