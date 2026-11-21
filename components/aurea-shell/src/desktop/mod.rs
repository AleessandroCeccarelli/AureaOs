// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Platform-independent desktop state shared by native shell surfaces.

mod dock;
mod launcher;
mod night_light;
mod panels;
mod workspace;

pub use dock::{DockMode, DockState};
pub use launcher::LauncherState;
pub use panels::{DesktopPanel, DesktopPanelState};
pub use night_light::{
    NightLightMode, NightLightSchedule, NightLightScheduleError, NightLightState,
};
pub use workspace::{WORKSPACE_COUNT, WorkspaceId, WorkspaceManager};

/// Desktop appearance preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DesktopTheme {
    #[default]
    Light,
    Dark,
}

/// State composed by the desktop shell and consumed by its presentation layer.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DesktopState {
    pub dock: DockState,
    pub launcher: LauncherState,
    pub workspaces: WorkspaceManager,
    pub panels: DesktopPanelState,
    pub theme: DesktopTheme,
    pub night_light: NightLightState,
}

impl DesktopState {
    /// Toggles the launcher without changing the current workspace.
    pub fn toggle_launcher(&mut self) {
        self.launcher.toggle();
    }

    /// Closes transient desktop panels.
    pub fn dismiss_panels(&mut self) {
        self.launcher.close();
    }

    /// Switches between the two supported appearance preferences.
    pub fn toggle_theme(&mut self) {
        self.theme = match self.theme {
            DesktopTheme::Light => DesktopTheme::Dark,
            DesktopTheme::Dark => DesktopTheme::Light,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::{DesktopState, DesktopTheme, NightLightMode};

    #[test]
    fn default_desktop_starts_with_light_theme_and_closed_launcher() {
        let state = DesktopState::default();
        assert_eq!(state.theme, DesktopTheme::Light);
        assert!(!state.launcher.is_open());
        assert!(!state.night_light.is_enabled());
    }

    #[test]
    fn dismissing_panels_closes_the_launcher() {
        let mut state = DesktopState::default();
        state.toggle_launcher();
        state.dismiss_panels();
        assert!(!state.launcher.is_open());
    }

    #[test]
    fn theme_toggle_is_reversible() {
        let mut state = DesktopState::default();
        state.toggle_theme();
        assert_eq!(state.theme, DesktopTheme::Dark);
        state.toggle_theme();
        assert_eq!(state.theme, DesktopTheme::Light);
    }

    #[test]
    fn night_light_preferences_are_part_of_desktop_state() {
        let mut state = DesktopState::default();
        state.night_light.set_enabled(true);
        state.night_light.set_mode(NightLightMode::CustomSchedule);
        assert!(state.night_light.is_enabled());
        assert_eq!(state.night_light.mode(), NightLightMode::CustomSchedule);
    }
}
