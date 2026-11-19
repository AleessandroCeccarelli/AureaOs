// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Software-managed keyboard layout contracts.

mod layout;

pub use layout::{
    KeyboardLayoutError, KeyboardLayoutHintSource, KeyboardLayoutId, KeyboardLayoutInfo,
    KeyboardLayoutManager, KeyboardLayoutSelection, KeyboardLayoutSuggestion,
};
