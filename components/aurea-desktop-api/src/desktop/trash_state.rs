// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Technology-independent visual state of the desktop Trash.

/// Semantic contents state used to select the desktop Trash artwork.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DesktopTrashState {
    /// No deleted items are present.
    #[default]
    Empty,
    /// The Trash contains items but is below its configured full threshold.
    PartiallyFull,
    /// The Trash has reached its configured full threshold.
    Full,
}
