// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Launcher panel state and search query ownership.

/// State for the application launcher panel.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LauncherState {
    open: bool,
    query: String,
}

impl LauncherState {
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn toggle(&mut self) {
        self.open = !self.open;
        if !self.open {
            self.query.clear();
        }
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.query.clear();
    }

    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
    }
}

#[cfg(test)]
mod tests {
    use super::LauncherState;

    #[test]
    fn launcher_opens_and_closes() {
        let mut launcher = LauncherState::default();
        launcher.open();
        assert!(launcher.is_open());
        launcher.close();
        assert!(!launcher.is_open());
    }

    #[test]
    fn closing_launcher_clears_search_query() {
        let mut launcher = LauncherState::default();
        launcher.open();
        launcher.set_query("impostazioni");
        launcher.close();
        assert!(launcher.query().is_empty());
    }

    #[test]
    fn opening_an_already_open_launcher_preserves_query() {
        let mut launcher = LauncherState::default();
        launcher.open();
        launcher.set_query("fides");
        launcher.open();
        assert_eq!(launcher.query(), "fides");
    }
}
