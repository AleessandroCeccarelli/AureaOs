// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 AureaOS project contributors
//
// Default configuration for the AureaOS desktop shell.

/// Basic configuration for the Aurea Shell.
pub struct ShellConfig {
    /// Name displayed by the shell.
    pub name: &'static str,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            name: "Aurea Shell",
        }
    }
}