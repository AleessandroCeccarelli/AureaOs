// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS test module relocated from production source.

use super::ShellConfig;

#[test]
fn default_config_has_expected_name() {
    let config = ShellConfig::default();

    assert_eq!(config.name, "Aurea Shell");
}
