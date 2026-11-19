// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Global desktop shortcut registry and resolution entry point.
//!
//! Application-level shortcuts remain owned by the focused Wayland client.

mod action;
mod workspace;

pub use action::ShortcutAction;
use smithay::input::keyboard::{Keysym, ModifiersState};

/// Resolves a registered desktop shortcut to its semantic action.
pub fn resolve_shortcut(keysym: Keysym, modifiers: ModifiersState) -> Option<ShortcutAction> {
    workspace::resolve(keysym.raw(), modifiers)
}
