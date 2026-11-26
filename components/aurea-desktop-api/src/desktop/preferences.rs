// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! User preferences for desktop presentation.

use super::DesktopItemId;

/// Optional rule used to group desktop files into visual stacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackGrouping {
    ByCategory,
    ByExtension,
}

/// Preferences that affect the desktop canvas, not the underlying files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopPreferences {
    show_trash: bool,
    stack_grouping: Option<StackGrouping>,
    excluded_from_stacks: Vec<DesktopItemId>,
}

impl DesktopPreferences {
    pub const fn show_trash(&self) -> bool {
        self.show_trash
    }
    pub fn set_show_trash(&mut self, show: bool) {
        self.show_trash = show;
    }
    pub const fn stack_grouping(&self) -> Option<StackGrouping> {
        self.stack_grouping
    }
    pub fn set_stack_grouping(&mut self, grouping: Option<StackGrouping>) {
        self.stack_grouping = grouping;
    }
    pub fn is_excluded_from_stacks(&self, id: &DesktopItemId) -> bool {
        self.excluded_from_stacks.contains(id)
    }
    pub fn set_excluded_from_stacks(&mut self, id: DesktopItemId, excluded: bool) {
        match (self.excluded_from_stacks.contains(&id), excluded) {
            (false, true) => self.excluded_from_stacks.push(id),
            (true, false) => self.excluded_from_stacks.retain(|item| item != &id),
            _ => {}
        }
    }
}

impl Default for DesktopPreferences {
    fn default() -> Self {
        Self {
            show_trash: true,
            stack_grouping: None,
            excluded_from_stacks: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DesktopPreferences, StackGrouping};
    use crate::desktop::DesktopItemId;

    #[test]
    fn stacks_are_disabled_by_default() {
        let preferences = DesktopPreferences::default();
        assert_eq!(preferences.stack_grouping(), None);
        assert!(preferences.show_trash());
    }

    #[test]
    fn excluded_items_can_be_reincluded() {
        let mut preferences = DesktopPreferences::default();
        let id = DesktopItemId::new("file:one").unwrap();
        preferences.set_excluded_from_stacks(id.clone(), true);
        assert!(preferences.is_excluded_from_stacks(&id));
        preferences.set_excluded_from_stacks(id.clone(), false);
        assert!(!preferences.is_excluded_from_stacks(&id));
        preferences.set_stack_grouping(Some(StackGrouping::ByExtension));
        assert_eq!(
            preferences.stack_grouping(),
            Some(StackGrouping::ByExtension)
        );
    }
}
