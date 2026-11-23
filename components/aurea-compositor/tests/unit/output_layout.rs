// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS test module relocated from production source.

use super::*;
#[test]
fn inserts_and_reads_positions() {
    let mut layout = OutputLayout::new();
    let position = OutputPosition::new(1920, 0);
    assert!(layout.insert(2, position));
    assert_eq!(layout.get(2), Some(position));
    assert_eq!(layout.len(), 1);
}
#[test]
fn rejects_duplicate_ids() {
    let mut layout = OutputLayout::new();
    assert!(layout.insert(1, OutputPosition::new(0, 0)));
    assert!(!layout.insert(1, OutputPosition::new(1920, 0)));
    assert_eq!(layout.len(), 1);
}
#[test]
fn updates_existing_positions_only() {
    let mut layout = OutputLayout::new();
    let position = OutputPosition::new(1920, 0);
    assert!(!layout.set_position(1, position));
    assert!(layout.insert(1, OutputPosition::default()));
    assert!(layout.set_position(1, position));
    assert_eq!(layout.get(1), Some(position));
}
