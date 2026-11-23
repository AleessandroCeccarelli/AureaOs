// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Floating desktop bar visibility policy.

/// User-selected dock visibility behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DockMode {
    /// Keep the dock visible unless a future layout policy needs to move it.
    #[default]
    Intelligent,
    /// Keep the dock visible.
    AlwaysVisible,
    /// Reveal the dock only when requested by pointer or keyboard focus.
    AutoHide,
}

/// Tracks dock visibility independently from the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DockState {
    mode: DockMode,
    keyboard_revealed: bool,
}

impl DockState {
    pub fn mode(&self) -> DockMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: DockMode) {
        self.mode = mode;
        if mode != DockMode::AutoHide {
            self.keyboard_revealed = false;
        }
    }

    pub fn reveal_for_keyboard(&mut self, revealed: bool) {
        self.keyboard_revealed = revealed;
    }

    /// Determines visibility from the selected policy and current interaction.
    pub fn is_visible(&self, pointer_at_screen_edge: bool, pointer_over_dock: bool) -> bool {
        match self.mode {
            DockMode::Intelligent | DockMode::AlwaysVisible => true,
            DockMode::AutoHide => {
                self.keyboard_revealed || pointer_at_screen_edge || pointer_over_dock
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DockMode, DockState};

    #[test]
    fn dock_is_visible_by_default() {
        assert!(DockState::default().is_visible(false, false));
    }

    #[test]
    fn auto_hide_requires_an_interaction() {
        let mut dock = DockState::default();
        dock.set_mode(DockMode::AutoHide);
        assert!(!dock.is_visible(false, false));
        assert!(dock.is_visible(true, false));
        assert!(dock.is_visible(false, true));
    }

    #[test]
    fn keyboard_reveal_keeps_auto_hidden_dock_visible() {
        let mut dock = DockState::default();
        dock.set_mode(DockMode::AutoHide);
        dock.reveal_for_keyboard(true);
        assert!(dock.is_visible(false, false));
    }
}
