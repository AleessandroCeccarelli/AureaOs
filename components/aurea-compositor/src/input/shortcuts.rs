// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Global keyboard shortcut definitions.

use smithay::input::keyboard::{Keysym, ModifiersState, keysyms};

/// Desktop actions triggered by global keyboard shortcuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutAction {
    /// Move to the previous virtual workspace.
    PreviousWorkspace,
    /// Move to the next virtual workspace.
    NextWorkspace,
}

/// Resolves supported global shortcuts without depending on an input backend.
///
/// Workspace navigation follows the documented Super + Ctrl + Left/Right
/// bindings. Extra modifiers are rejected so the compositor does not consume
/// combinations intended for the focused application.
pub fn resolve_shortcut(keysym: Keysym, modifiers: ModifiersState) -> Option<ShortcutAction> {
    if !modifiers.logo || !modifiers.ctrl || modifiers.alt || modifiers.shift {
        return None;
    }

    match keysym.raw() {
        keysyms::KEY_Left => Some(ShortcutAction::PreviousWorkspace),
        keysyms::KEY_Right => Some(ShortcutAction::NextWorkspace),
        _ => None,
    }
}
