// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli

//! Logical content state of the desktop Trash.

/// Visual state derived from the Trash service's actual item count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrashContentsState {
    /// No deleted items are present.
    #[default]
    Empty,
    /// The Trash contains items but is not considered full.
    PartiallyFull,
    /// The Trash has reached the configured full-state threshold.
    Full,
}

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
