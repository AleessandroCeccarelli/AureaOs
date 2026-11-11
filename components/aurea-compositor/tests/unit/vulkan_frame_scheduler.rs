// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use aurea_compositor::vulkan::next_frame_slot_for_test;

#[test]
fn frame_ring_wraps_after_last_slot() {
    assert_eq!(next_frame_slot_for_test(0, 2), 1);
    assert_eq!(next_frame_slot_for_test(1, 2), 0);
}

#[test]
fn frame_ring_supports_three_slots() {
    assert_eq!(next_frame_slot_for_test(0, 3), 1);
    assert_eq!(next_frame_slot_for_test(1, 3), 2);
    assert_eq!(next_frame_slot_for_test(2, 3), 0);
}

#[test]
fn empty_frame_ring_is_stable() {
    assert_eq!(next_frame_slot_for_test(0, 0), 0);
}
