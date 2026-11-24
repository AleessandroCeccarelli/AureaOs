// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Owned desktop-entry data captured for one render frame.

use aurea_desktop_api::desktop::{DesktopItem, DesktopManager};

/// Immutable, frame-owned copy of the desktop entries.
///
/// This is deliberately separate from the compositor's small Copy snapshot
/// for desktop appearance and wallpaper state. The item collection owns its
/// data so a render frame never borrows the mutable model.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DesktopItemsSnapshot {
    items: Vec<DesktopItem>,
}

impl DesktopItemsSnapshot {
    /// Captures the current desktop entries in their model-defined visual order.
    pub fn capture(manager: &DesktopManager) -> Self {
        Self { items: manager.items().to_vec() }
    }

    /// Returns the immutable entries in their captured order.
    pub fn items(&self) -> &[DesktopItem] {
        &self.items
    }

    /// Returns the number of captured desktop entries.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns whether the captured desktop contains no entries.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::DesktopItemsSnapshot;
    use aurea_desktop_api::desktop::DesktopManager;

    #[test]
    fn captures_default_system_trash_entry() {
        let manager = DesktopManager::default();
        let snapshot = DesktopItemsSnapshot::capture(&manager);
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot.items()[0].id().as_str(), "system:trash");
    }

    #[test]
    fn snapshot_is_owned_and_does_not_change_with_model() {
        let mut manager = DesktopManager::default();
        let snapshot = DesktopItemsSnapshot::capture(&manager);
        let trash_id = snapshot.items()[0].id().clone();
        manager.set_item_position(
            &trash_id,
            aurea_desktop_api::desktop::DesktopPosition::new(120, 80),
        ).unwrap();
        assert_eq!(
            snapshot.items()[0].position(),
            aurea_desktop_api::desktop::DesktopPosition::new(32, 32)
        );
    }
}
