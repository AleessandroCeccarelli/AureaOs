
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use aurea_compositor::output::{
    DisplayOutput,
    OutputId,
    OutputMode,
    OutputRegistry,
    layout::{OutputLayout, OutputPosition},
    manager::{OutputManager, OutputManagerError, OutputReconciliation},
};

#[path = "unit/frame_scheduler.rs"]
mod frame_scheduler;

#[path = "unit/output.rs"]
mod output;

#[path = "unit/output_layout.rs"]
mod output_layout;

#[path = "unit/output_manager.rs"]
mod output_manager;

#[path = "unit/compositor_output_reconciliation.rs"]
mod compositor_output_reconciliation;

#[cfg(all(feature = "drm", feature = "vulkan"))]
#[path = "unit/vulkan_drm_compatibility.rs"]
mod vulkan_drm_compatibility;

#[cfg(all(feature = "drm", feature = "vulkan"))]
#[path = "unit/vulkan_buffer_allocator.rs"]
mod vulkan_buffer_allocator;

#[cfg(all(feature = "drm", feature = "vulkan"))]
#[path = "unit/vulkan_dmabuf_layout.rs"]
mod vulkan_dmabuf_layout;

#[cfg(feature = "vulkan")]
#[path = "unit/vulkan_device_extensions.rs"]
mod vulkan_device_extensions;

#[cfg(feature = "vulkan")]
#[path = "unit/vulkan_memory_type.rs"]
mod vulkan_memory_type;

#[cfg(feature = "vulkan")]
#[path = "unit/vulkan_frame_scheduler.rs"]
mod vulkan_frame_scheduler;

#[cfg(feature = "vulkan")]
#[path = "unit/vulkan_shader.rs"]
mod vulkan_shader;

#[cfg(feature = "vulkan")]
#[path = "unit/vulkan_clear_pass.rs"]
mod vulkan_clear_pass;

#[cfg(feature = "vulkan")]
#[path = "unit/vulkan_readback.rs"]
mod vulkan_readback;
