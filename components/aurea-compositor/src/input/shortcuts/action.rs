// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Semantic actions produced by the desktop shortcut resolver.

/// Desktop action triggered by a registered global shortcut.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutAction {
    /// Move to the previous virtual workspace.
    PreviousWorkspace,
    /// Move to the next virtual workspace.
    NextWorkspace,
    /// Open the desktop overview.
    ShowOverview,
    /// Open the overview of all application windows.
    ShowAppExpose,
}
