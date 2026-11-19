// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Product-level limits shared by all display-management components.

/// Maximum number of display outputs AureaOS may keep active simultaneously.
///
/// This is an AureaOS product policy, not a hardware capability guarantee.
pub const MAX_ACTIVE_DISPLAYS: usize = 4;
