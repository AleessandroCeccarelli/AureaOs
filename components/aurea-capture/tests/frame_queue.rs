// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Capture Core tests.

use aurea_capture::{
    CaptureFrame, CaptureFrameQueue, CapturePixelFormat, CaptureQueueError, CaptureTimestamp,
};

fn frame(timestamp: u64, fill: u8) -> CaptureFrame {
    CaptureFrame::new(
        2,
        2,
        8,
        CapturePixelFormat::Rgba8,
        CaptureTimestamp::from_nanos(timestamp),
        vec![fill; 16],
    )
    .unwrap()
}

#[test]
fn frame_rejects_zero_dimensions() {
    assert!(CaptureFrame::new(
        0,
        2,
        8,
        CapturePixelFormat::Rgba8,
        CaptureTimestamp::from_nanos(0),
        vec![0; 16],
    )
    .is_err());
}

#[test]
fn frame_rejects_short_stride_and_incorrect_buffer_length() {
    assert!(CaptureFrame::new(
        2,
        2,
        4,
        CapturePixelFormat::Rgba8,
        CaptureTimestamp::from_nanos(0),
        vec![0; 8],
    )
    .is_err());

    assert!(CaptureFrame::new(
        2,
        2,
        8,
        CapturePixelFormat::Rgba8,
        CaptureTimestamp::from_nanos(0),
        vec![0; 15],
    )
    .is_err());
}

#[test]
fn frame_preserves_format_timestamp_and_pixels() {
    let captured = frame(1234, 0x7f);

    assert_eq!(captured.format(), CapturePixelFormat::Rgba8);
    assert_eq!(captured.timestamp().as_nanos(), 1234);
    assert_eq!(captured.pixels(), &[0x7f; 16]);
}

#[test]
fn queue_rejects_zero_capacity() {
    assert_eq!(
        CaptureFrameQueue::new(0, 1024).err(),
        Some(CaptureQueueError::ZeroCapacity)
    );
}

#[test]
fn queue_is_bounded_and_counts_dropped_frames() {
    let mut queue = CaptureFrameQueue::new(2, 32).unwrap();

    queue.try_push(frame(1, 1)).unwrap();
    queue.try_push(frame(2, 2)).unwrap();
    assert!(queue.try_push(frame(3, 3)).is_err());

    assert_eq!(queue.len(), 2);
    assert_eq!(queue.dropped_frames(), 1);
    assert_eq!(queue.queued_bytes(), 32);
}

#[test]
fn queue_preserves_frame_order_and_releases_accounted_bytes() {
    let mut queue = CaptureFrameQueue::new(2).unwrap();

    queue.try_push(frame(1, 1)).unwrap();
    queue.try_push(frame(2, 2)).unwrap();

    assert_eq!(queue.pop().unwrap().timestamp().as_nanos(), 1);
    assert_eq!(queue.queued_bytes(), 16);
    assert_eq!(queue.pop().unwrap().timestamp().as_nanos(), 2);
    assert!(queue.is_empty());
    assert_eq!(queue.queued_bytes(), 0);
}
