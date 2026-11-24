// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Owns the desktop entry collection and derives optional visual stacks.

use super::{
    stacks::build_stacks, DesktopItem, DesktopItemId, DesktopItemKind, DesktopPosition,
    DesktopPreferences, DesktopStack, DesktopTrashState, StackGrouping,
};

/// Errors returned when changing the desktop entry collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopManagerError {
    DuplicateItem,
    MissingItem,
    SystemItem,
}

/// Mutable model for one user's desktop canvas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopManager {
    items: Vec<DesktopItem>,
    preferences: DesktopPreferences,
    trash_state: DesktopTrashState,
}

impl DesktopManager {
    pub fn items(&self) -> &[DesktopItem] { &self.items }
    pub const fn preferences(&self) -> &DesktopPreferences { &self.preferences }
    pub fn preferences_mut(&mut self) -> &mut DesktopPreferences { &mut self.preferences }
    pub const fn trash_state(&self) -> DesktopTrashState { self.trash_state }

    /// Updates the visual state supplied by the Trash service.
    pub fn set_trash_state(&mut self, state: DesktopTrashState) {
        self.trash_state = state;
    }

    pub fn add_item(&mut self, item: DesktopItem) -> Result<(), DesktopManagerError> {
        if self.items.iter().any(|existing| existing.id() == item.id()) {
            return Err(DesktopManagerError::DuplicateItem);
        }
        self.items.push(item);
        Ok(())
    }

    pub fn remove_item(&mut self, id: &DesktopItemId) -> Result<DesktopItem, DesktopManagerError> {
        let index = self.items.iter().position(|item| item.id() == id)
            .ok_or(DesktopManagerError::MissingItem)?;
        if matches!(self.items[index].kind(), DesktopItemKind::Trash) {
            return Err(DesktopManagerError::SystemItem);
        }
        Ok(self.items.remove(index))
    }

    pub fn set_item_position(
        &mut self,
        id: &DesktopItemId,
        position: DesktopPosition,
    ) -> Result<(), DesktopManagerError> {
        let item = self.items.iter_mut().find(|item| item.id() == id)
            .ok_or(DesktopManagerError::MissingItem)?;
        item.set_position(position);
        Ok(())
    }

    pub fn stacks(&self) -> Vec<DesktopStack> {
        let Some(grouping) = self.preferences.stack_grouping() else { return Vec::new() };
        build_stacks(&self.items, grouping, |id| self.preferences.is_excluded_from_stacks(id))
    }

    pub fn set_stack_grouping(&mut self, grouping: Option<StackGrouping>) {
        self.preferences.set_stack_grouping(grouping);
    }
}

impl Default for DesktopManager {
    fn default() -> Self {
        let trash = DesktopItem::new(
            DesktopItemId::new("system:trash").expect("static desktop item ID is valid"),
            "Cestino",
            DesktopItemKind::Trash,
            DesktopPosition::new(32, 32),
        );
        Self {
            items: vec![trash],
            preferences: DesktopPreferences::default(),
            trash_state: DesktopTrashState::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DesktopManager, DesktopManagerError};
    use crate::desktop::{
        DesktopItem, DesktopItemId, DesktopItemKind, DesktopPosition, DesktopTrashState,
        StackGrouping,
    };

    fn file(id: &str, name: &str) -> DesktopItem {
        DesktopItem::new(
            DesktopItemId::new(id).unwrap(),
            name,
            DesktopItemKind::File { uri: format!("file:///home/user/{name}") },
            DesktopPosition::default(),
        )
    }

    #[test]
    fn default_desktop_contains_only_the_trash() {
        let manager = DesktopManager::default();
        assert_eq!(manager.items().len(), 1);
        assert!(matches!(manager.items()[0].kind(), DesktopItemKind::Trash));
        assert!(manager.stacks().is_empty());
    }

    #[test]
    fn rejects_duplicate_entries_and_preserves_system_trash() {
        let mut manager = DesktopManager::default();
        let item = file("file:one", "notes.txt");
        manager.add_item(item.clone()).unwrap();
        assert_eq!(manager.add_item(item), Err(DesktopManagerError::DuplicateItem));
        let trash_id = DesktopItemId::new("system:trash").unwrap();
        assert_eq!(manager.remove_item(&trash_id), Err(DesktopManagerError::SystemItem));
    }

    #[test]
    fn trash_visual_state_is_shared_with_desktop_snapshots() {
        let mut manager = DesktopManager::default();
        assert_eq!(manager.trash_state(), DesktopTrashState::Empty);

        manager.set_trash_state(DesktopTrashState::Full);

        assert_eq!(manager.trash_state(), DesktopTrashState::Full);
    }

    #[test]
    fn stack_grouping_is_optional_and_derived() {
        let mut manager = DesktopManager::default();
        manager.add_item(file("one", "a.pdf")).unwrap();
        manager.add_item(file("two", "b.pdf")).unwrap();
        assert!(manager.stacks().is_empty());
        manager.set_stack_grouping(Some(StackGrouping::ByExtension));
        assert_eq!(manager.stacks().len(), 1);
        manager.set_stack_grouping(None);
        assert!(manager.stacks().is_empty());
        assert_eq!(manager.items().len(), 3);
    }
}
