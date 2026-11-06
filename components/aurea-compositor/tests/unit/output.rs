// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS test module relocated from production source.

use super::*;
fn mode(width: u32, height: u32, refresh: u32) -> OutputMode {
    OutputMode::new(width, height, refresh).unwrap()
}
fn sample_output() -> DisplayOutput {
    let hd = mode(1920, 1080, 60_000);
    DisplayOutput::new(1, "HDMI-A-1", vec![hd], hd, 1).unwrap()
}
#[test]
fn rejects_invalid_modes() {
    assert!(OutputMode::new(0, 1080, 60_000).is_none());
    assert!(OutputMode::new(1920, 1080, 0).is_none());
}
#[test]
fn rejects_output_with_unsupported_current_mode() {
    let supported = mode(1920, 1080, 60_000);
    let unsupported = mode(1280, 720, 60_000);
    assert!(DisplayOutput::new(1, "HDMI-A-1", vec![supported], unsupported, 1,).is_none());
}
#[test]
fn registry_rejects_duplicate_ids() {
    let mut registry = OutputRegistry::new();
    assert!(registry.register(sample_output()));
    assert!(!registry.register(sample_output()));
    assert_eq!(registry.len(), 1);
}
#[test]
fn output_mode_and_scale_can_be_updated() {
    let mut output = sample_output();
    let new_mode = mode(1280, 720, 60_000);
    assert!(!output.set_mode(new_mode));
    assert!(!output.set_scale(0));
    assert!(output.set_scale(2));
    assert_eq!(output.scale(), 2);
}
