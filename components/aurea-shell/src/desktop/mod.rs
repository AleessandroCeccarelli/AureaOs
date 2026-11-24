// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Platform-independent desktop state shared by native shell surfaces.

mod dock;
mod launcher;
mod night_light;
mod panels;
mod trash;
mod trash_artwork;
mod workspace;

pub use dock::{DockMode, DockState};
pub use launcher::LauncherState;
pub use night_light::{
    NightLightMode, NightLightSchedule, NightLightScheduleError, NightLightState,
};
pub use panels::{DesktopPanel, DesktopPanelState};
pub use trash::TrashContentsState;
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
    pub trash_contents: TrashContentsState,
    selected_item_id: Option<String>,
}

impl DesktopState {
    /// Returns the identifier of the currently selected desktop item.
    pub fn selected_item_id(&self) -> Option<&str> {
        self.selected_item_id.as_deref()
    }

    /// Selects one desktop item by its stable identifier.
    pub fn select_item(&mut self, id: impl Into<String>) {
        self.selected_item_id = Some(id.into());
    }

    /// Clears the current desktop item selection.
    pub fn clear_item_selection(&mut self) {
        self.selected_item_id = None;
    }

    /// Toggles the launcher without changing the current workspace.
    pub fn toggle_launcher(&mut self) {
        self.launcher.toggle();
    }

    /// Closes transient desktop panels.
    pub fn dismiss_panels(&mut self) {
        self.launcher.close();
        self.panels.close();
    }

    /// Switches between the two supported appearance preferences.
    pub fn toggle_theme(&mut self) {
        self.theme = match self.theme {
            DesktopTheme::Light => DesktopTheme::Dark,
            DesktopTheme::Dark => DesktopTheme::Light,
        };
    }

    /// Selects the Trash artwork for the current theme and content state.
    pub fn trash_artwork_svg(&self) -> &'static [u8] {
        trash_artwork::svg(self.theme, self.trash_contents)
    }

    /// Updates the visual Trash state supplied by the Trash service.
    pub fn set_trash_contents(&mut self, state: TrashContentsState) {
        self.trash_contents = state;
    }
}

#[cfg(test)]
mod tests {
    use super::{DesktopState, DesktopTheme, NightLightMode, TrashContentsState};

    #[test]
    fn default_desktop_starts_with_light_theme_empty_trash_and_closed_launcher() {
        let state = DesktopState::default();
        assert_eq!(state.theme, DesktopTheme::Light);
        assert_eq!(state.trash_contents, TrashContentsState::Empty);
        assert!(!state.launcher.is_open());
        assert!(!state.night_light.is_enabled());
        assert!(std::str::from_utf8(state.trash_artwork_svg())
            .unwrap()
            .contains("— empty"));
    }

    #[test]
    fn dismissing_panels_closes_launcher_and_transient_panel() {
        let mut state = DesktopState::default();
        state.toggle_launcher();
        state.panels.show(super::DesktopPanel::Overview);
        state.dismiss_panels();
        assert!(!state.launcher.is_open());
        assert!(!state.panels.is_open());
    }

    #[test]
    fn theme_toggle_selects_the_matching_trash_artwork() {
        let mut state = DesktopState::default();
        let light = state.trash_artwork_svg();
        state.toggle_theme();
        assert_eq!(state.theme, DesktopTheme::Dark);
        assert_ne!(state.trash_artwork_svg(), light);
    }

    #[test]
    fn trash_state_selects_the_matching_artwork() {
        let mut state = DesktopState::default();
        state.set_trash_contents(TrashContentsState::Full);
        assert!(std::str::from_utf8(state.trash_artwork_svg())
            .unwrap()
            .contains("overfilled pile"));
    }

    #[test]
    fn desktop_item_selection_can_be_set_and_cleared() {
        let mut state = DesktopState::default();
        assert_eq!(state.selected_item_id(), None);

        state.select_item("system:trash");
        assert_eq!(state.selected_item_id(), Some("system:trash"));

        state.clear_item_selection();
        assert_eq!(state.selected_item_id(), None);
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
