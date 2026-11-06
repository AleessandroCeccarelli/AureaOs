// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Translates shell events into runtime commands.

use crate::event::ShellEvent;

/// Commands produced by the event dispatcher.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeCommand {
    LockSession,
    UnlockSession,
    Shutdown,
}

/// Converts a shell event into a runtime command.
pub fn dispatch(event: ShellEvent) -> RuntimeCommand {
    match event {
        ShellEvent::LockRequested => RuntimeCommand::LockSession,
        ShellEvent::UnlockRequested => RuntimeCommand::UnlockSession,
        ShellEvent::ShutdownRequested => RuntimeCommand::Shutdown,
    }
}


#[cfg(test)]
#[path = "../tests/unit/dispatcher.rs"]
mod tests;
