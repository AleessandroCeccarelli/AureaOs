// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Validated DMA-BUF metadata used at the Vulkan import boundary.

use std::{error::Error, fmt};

use smithay::backend::allocator::{
    Buffer,
    dmabuf::{Dmabuf, MAX_PLANES},
};

/// Layout metadata for one DMA-BUF memory plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DmabufPlaneLayout {
    /// Byte offset of this plane in its DMA-BUF allocation.
    pub offset: u32,
    /// Number of bytes between adjacent rows in this plane.
    pub stride: u32,
}

/// Immutable metadata snapshot for importing a DMA-BUF into Vulkan.
///
/// The snapshot deliberately does not duplicate file descriptors. The
/// originating Dmabuf must remain alive while an importer consumes its
/// borrowed plane handles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DmabufImportLayout {
    width: u32,
    height: u32,
    format: smithay::backend::allocator::Format,
    planes: Vec<DmabufPlaneLayout>,
    y_inverted: bool,
}

/// Errors found while validating DMA-BUF metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmabufLayoutError {
    /// DMA-BUF dimensions must be positive.
    InvalidDimensions { width: i32, height: i32 },
    /// A DMA-BUF must contain between one and MAX_PLANES memory planes.
    InvalidPlaneCount { count: usize },
    /// A memory plane cannot have a zero row stride.
    ZeroStride { plane: usize },
    /// Plane offset plus stride overflowed.
    PlaneLayoutOverflow { plane: usize },
}

impl fmt::Display for DmabufLayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions { width, height } => {
                write!(formatter, "Invalid DMA-BUF dimensions: {width}x{height}")
            }
            Self::InvalidPlaneCount { count } => {
                write!(formatter, "Unsupported DMA-BUF memory-plane count: {count}")
            }
            Self::ZeroStride { plane } => {
                write!(formatter, "DMA-BUF memory plane {plane} has a zero stride")
            }
            Self::PlaneLayoutOverflow { plane } => {
                write!(formatter, "DMA-BUF memory plane {plane} layout overflows")
            }
        }
    }
}

impl Error for DmabufLayoutError {}

impl DmabufImportLayout {
    /// Validates and snapshots dimensions, format, modifier and per-plane layout.
    pub fn from_dmabuf(dmabuf: &Dmabuf) -> Result<Self, DmabufLayoutError> {
        let size = dmabuf.size();
        if size.w <= 0 || size.h <= 0 {
            return Err(DmabufLayoutError::InvalidDimensions {
                width: size.w,
                height: size.h,
            });
        }

        let offsets = dmabuf.offsets().collect::<Vec<_>>();
        let strides = dmabuf.strides().collect::<Vec<_>>();
        let handle_count = dmabuf.handles().count();
        let plane_count = dmabuf.num_planes();

        if plane_count == 0
            || plane_count > MAX_PLANES
            || offsets.len() != plane_count
            || strides.len() != plane_count
            || handle_count != plane_count
        {
            return Err(DmabufLayoutError::InvalidPlaneCount { count: plane_count });
        }

        let mut planes = Vec::with_capacity(plane_count);
        for (index, (offset, stride)) in offsets.into_iter().zip(strides).enumerate() {
            if stride == 0 {
                return Err(DmabufLayoutError::ZeroStride { plane: index });
            }
            offset
                .checked_add(stride)
                .ok_or(DmabufLayoutError::PlaneLayoutOverflow { plane: index })?;
            planes.push(DmabufPlaneLayout { offset, stride });
        }

        Ok(Self {
            width: size.w as u32,
            height: size.h as u32,
            format: dmabuf.format(),
            planes,
            y_inverted: dmabuf.y_inverted(),
        })
    }

    /// Buffer width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }
    /// Buffer height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }
    /// DRM FourCC and modifier pair.
    pub fn format(&self) -> smithay::backend::allocator::Format {
        self.format
    }
    /// Per-plane offsets and row strides in plane-index order.
    pub fn planes(&self) -> &[DmabufPlaneLayout] {
        &self.planes
    }
    /// Whether the buffer content is vertically inverted.
    pub fn y_inverted(&self) -> bool {
        self.y_inverted
    }
}
