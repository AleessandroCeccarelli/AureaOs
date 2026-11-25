// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! State for transient desktop presentation surfaces.

use aurea_desktop_api::desktop_interaction::DesktopPosition;

/// A transient desktop surface requested by an input or shell action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopPanel {
    Overview,
    AppExpose,
    ContextMenu,
}

/// Tracks which transient desktop surface is currently requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DesktopPanelState {
    active: Option<DesktopPanel>,
    context_menu_position: Option<DesktopPosition>,
}

impl DesktopPanelState {
    /// Returns the active transient surface, if any.
    pub fn active(&self) -> Option<DesktopPanel> {
        self.active
    }

    /// Opens a transient surface, replacing any other active surface.
    pub fn show(&mut self, panel: DesktopPanel) {
        self.active = Some(panel);
        if panel != DesktopPanel::ContextMenu {
            self.context_menu_position = None;
        }
    }

    /// Opens the desktop context menu at the pointer's logical position.
    pub fn show_context_menu(&mut self, position: DesktopPosition) {
        self.active = Some(DesktopPanel::ContextMenu);
        self.context_menu_position = Some(position);
    }

    /// Returns the position requested for the active context menu.
    pub fn context_menu_position(&self) -> Option<DesktopPosition> {
        self.context_menu_position
    }

    /// Closes the active transient surface.
    pub fn close(&mut self) {
        self.active = None;
        self.context_menu_position = None;
    }

    /// Reports whether a transient surface is currently requested.
    pub fn is_open(&self) -> bool {
        self.active.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::{DesktopPanel, DesktopPanelState};
    use aurea_desktop_api::desktop_interaction::DesktopPosition;

    #[test]
    fn showing_a_panel_replaces_the_previous_panel() {
        let mut panels = DesktopPanelState::default();
        panels.show(DesktopPanel::Overview);
        panels.show(DesktopPanel::ContextMenu);

        assert_eq!(panels.active(), Some(DesktopPanel::ContextMenu));
    }

    #[test]
    fn context_menu_retains_pointer_position_and_close_clears_it() {
        let mut panels = DesktopPanelState::default();
        let position = DesktopPosition { x: 640, y: 360 };

        panels.show_context_menu(position);
        assert_eq!(panels.active(), Some(DesktopPanel::ContextMenu));
        assert_eq!(panels.context_menu_position(), Some(position));

        panels.close();
        assert_eq!(panels.context_menu_position(), None);
    }

    #[test]
    fn closing_panels_clears_the_active_surface() {
        let mut panels = DesktopPanelState::default();
        panels.show(DesktopPanel::Overview);
        panels.close();

        assert_eq!(panels.active(), None);
        assert!(!panels.is_open());
    }
}
