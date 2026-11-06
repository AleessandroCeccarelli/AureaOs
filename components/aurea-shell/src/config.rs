// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Default configuration and unit tests for the AureaOS desktop shell.

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

#[cfg(test)]
mod tests {
    use super::ShellConfig;

    #[test]
    fn default_config_has_expected_name() {
        let config = ShellConfig::default();

        assert_eq!(config.name, "Aurea Shell");
    }
}
