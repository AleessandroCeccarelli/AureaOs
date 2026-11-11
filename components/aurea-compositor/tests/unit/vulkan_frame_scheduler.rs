// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use aurea_compositor::vulkan::VulkanFrameScheduler;

#[test]
fn frame_ring_wraps_after_last_slot() {
    assert_eq!(VulkanFrameScheduler::next_slot_index(0, 2), 1);
    assert_eq!(VulkanFrameScheduler::next_slot_index(1, 2), 0);
}

#[test]
fn frame_ring_supports_three_slots() {
    assert_eq!(VulkanFrameScheduler::next_slot_index(0, 3), 1);
    assert_eq!(VulkanFrameScheduler::next_slot_index(1, 3), 2);
    assert_eq!(VulkanFrameScheduler::next_slot_index(2, 3), 0);
}

#[test]
fn empty_frame_ring_is_stable() {
    assert_eq!(VulkanFrameScheduler::next_slot_index(0, 0), 0);
}

#[test]
fn out_of_range_index_is_normalized() {
    assert_eq!(VulkanFrameScheduler::next_slot_index(4, 3), 2);
    assert_eq!(VulkanFrameScheduler::next_slot_index(6, 3), 1);
}

#[test]
fn maximum_index_does_not_overflow() {
    assert_eq!(VulkanFrameScheduler::next_slot_index(usize::MAX, 2), 0);
}
