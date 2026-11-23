// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Hardware-independent tests for DMA-BUF import metadata validation.

use std::fs::File;

use aurea_compositor::vulkan::{DmabufImportLayout, DmabufLayoutError};
use smithay::backend::allocator::{
    Fourcc, Modifier,
    dmabuf::{Dmabuf, DmabufFlags},
};

fn build_dmabuf(offset: u32, stride: u32) -> Dmabuf {
    let mut builder = Dmabuf::builder(
        (128, 64),
        Fourcc::Argb8888,
        Modifier::Linear,
        DmabufFlags::empty(),
    );
    let fd = File::open("/dev/null").expect("open test fd").into();
    assert!(builder.add_plane(fd, 0, offset, stride));
    builder.build().expect("one-plane DMA-BUF should build")
}

#[test]
fn snapshots_dimensions_format_and_plane_layout() {
    let dmabuf = build_dmabuf(16, 512);
    let layout = DmabufImportLayout::from_dmabuf(&dmabuf).expect("valid layout");
    assert_eq!(layout.width(), 128);
    assert_eq!(layout.height(), 64);
    assert_eq!(layout.format().code, Fourcc::Argb8888);
    assert_eq!(layout.format().modifier, Modifier::Linear);
    assert_eq!(layout.planes().len(), 1);
    assert_eq!(layout.planes()[0].offset, 16);
    assert_eq!(layout.planes()[0].stride, 512);
}

#[test]
fn rejects_zero_plane_stride() {
    let dmabuf = build_dmabuf(0, 0);
    assert_eq!(
        DmabufImportLayout::from_dmabuf(&dmabuf),
        Err(DmabufLayoutError::ZeroStride { plane: 0 })
    );
}

#[test]
fn rejects_plane_layout_overflow() {
    let dmabuf = build_dmabuf(u32::MAX, 1);
    assert_eq!(
        DmabufImportLayout::from_dmabuf(&dmabuf),
        Err(DmabufLayoutError::PlaneLayoutOverflow { plane: 0 })
    );
}
