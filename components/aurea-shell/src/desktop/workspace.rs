// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Four-workspace navigation model.

/// AureaOS currently defines four virtual workspaces.
pub const WORKSPACE_COUNT: u8 = 4;

/// Zero-based workspace identifier.
pub type WorkspaceId = u8;

/// Owns the active virtual workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceManager {
    active: WorkspaceId,
}

impl WorkspaceManager {
    pub fn active(&self) -> WorkspaceId {
        self.active
    }

    /// Selects a workspace, returning false when the identifier is invalid.
    pub fn switch_to(&mut self, workspace: WorkspaceId) -> bool {
        if workspace >= WORKSPACE_COUNT {
            return false;
        }
        self.active = workspace;
        true
    }

    pub fn next(&mut self) {
        self.active = (self.active + 1) % WORKSPACE_COUNT;
    }

    pub fn previous(&mut self) {
        self.active = (self.active + WORKSPACE_COUNT - 1) % WORKSPACE_COUNT;
    }
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self { active: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::{WORKSPACE_COUNT, WorkspaceManager};

    #[test]
    fn starts_on_first_workspace() {
        assert_eq!(WorkspaceManager::default().active(), 0);
    }

    #[test]
    fn wraps_forward_and_backward() {
        let mut workspaces = WorkspaceManager::default();
        workspaces.previous();
        assert_eq!(workspaces.active(), WORKSPACE_COUNT - 1);
        workspaces.next();
        assert_eq!(workspaces.active(), 0);
    }

    #[test]
    fn rejects_out_of_range_workspace() {
        let mut workspaces = WorkspaceManager::default();
        assert!(!workspaces.switch_to(WORKSPACE_COUNT));
        assert_eq!(workspaces.active(), 0);
    }
}
