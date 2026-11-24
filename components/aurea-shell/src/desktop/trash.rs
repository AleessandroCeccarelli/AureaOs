// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Shared semantic content state of the desktop Trash.

pub use aurea_desktop_api::desktop::DesktopTrashState as TrashContentsState;

#[cfg(test)]
mod tests {
    use super::TrashContentsState;

    #[test]
    fn defaults_to_empty() {
        assert_eq!(TrashContentsState::default(), TrashContentsState::Empty);
    }

    #[test]
    fn states_are_distinct() {
        assert_ne!(TrashContentsState::Empty, TrashContentsState::PartiallyFull);
        assert_ne!(TrashContentsState::PartiallyFull, TrashContentsState::Full);
    }
}
