// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Shell events.

/// Events handled by the shell runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellEvent {
    LockRequested,
    UnlockRequested,
    ShutdownRequested,
}
