// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Keyboard bindings for virtual workspace navigation.

use smithay::input::keyboard::{ModifiersState, keysyms};

use super::action::ShortcutAction;

/// Resolves Super + Ctrl + Left/Right without consuming modified variants.
pub(super) fn resolve(keysym: u32, modifiers: ModifiersState) -> Option<ShortcutAction> {
    if !modifiers.logo || !modifiers.ctrl || modifiers.alt || modifiers.shift {
        return None;
    }

    match keysym {
        keysyms::KEY_Left => Some(ShortcutAction::PreviousWorkspace),
        keysyms::KEY_Right => Some(ShortcutAction::NextWorkspace),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use smithay::input::keyboard::{ModifiersState, keysyms};

    use super::resolve;
    use crate::input::shortcuts::ShortcutAction;

    #[test]
    fn resolves_only_documented_workspace_bindings() {
        let mut modifiers = ModifiersState::default();
        modifiers.logo = true;
        modifiers.ctrl = true;

        assert_eq!(
            resolve(keysyms::KEY_Left, modifiers),
            Some(ShortcutAction::PreviousWorkspace)
        );
        assert_eq!(
            resolve(keysyms::KEY_Right, modifiers),
            Some(ShortcutAction::NextWorkspace)
        );
        assert_eq!(resolve(keysyms::KEY_Up, modifiers), None);
    }

    #[test]
    fn rejects_missing_or_additional_modifiers() {
        let mut modifiers = ModifiersState::default();
        modifiers.logo = true;
        assert_eq!(resolve(keysyms::KEY_Left, modifiers), None);

        modifiers.ctrl = true;
        modifiers.shift = true;
        assert_eq!(resolve(keysyms::KEY_Left, modifiers), None);

        modifiers.shift = false;
        modifiers.alt = true;
        assert_eq!(resolve(keysyms::KEY_Left, modifiers), None);
    }
}
