// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Keyboard bindings for virtual workspace navigation.

use smithay::input::keyboard::{Keysym, ModifiersState, keysyms};

use super::action::ShortcutAction;

/// Resolves Super + Ctrl + Left/Right without consuming modified variants.
pub(super) fn resolve(
    keysym: Keysym,
    modifiers: ModifiersState,
) -> Option<ShortcutAction> {
    if !modifiers.logo || !modifiers.ctrl || modifiers.alt || modifiers.shift {
        return None;
    }

    match keysym.raw() {
        keysyms::KEY_Left => Some(ShortcutAction::PreviousWorkspace),
        keysyms::KEY_Right => Some(ShortcutAction::NextWorkspace),
        _ => None,
    }
}
