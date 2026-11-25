// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! State for transient desktop presentation surfaces.

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
}

impl DesktopPanelState {
    /// Returns the active transient surface, if any.
    pub fn active(&self) -> Option<DesktopPanel> {
        self.active
    }

    /// Opens a transient surface, replacing any other active surface.
    pub fn show(&mut self, panel: DesktopPanel) {
        self.active = Some(panel);
    }

    /// Closes the active transient surface.
    pub fn close(&mut self) {
        self.active = None;
    }

    /// Reports whether a transient surface is currently requested.
    pub fn is_open(&self) -> bool {
        self.active.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::{DesktopPanel, DesktopPanelState};

    #[test]
    fn showing_a_panel_replaces_the_previous_panel() {
        let mut panels = DesktopPanelState::default();
        panels.show(DesktopPanel::Overview);
        panels.show(DesktopPanel::ContextMenu);

        assert_eq!(panels.active(), Some(DesktopPanel::ContextMenu));
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
