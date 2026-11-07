// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Wayland compositor component for AureaOS.

pub mod compositor;
pub mod gpu;
pub mod server;
mod window_manager;
mod xdg_shell;

pub mod output;
pub mod render;
pub mod renderer;

#[cfg(feature = "drm")]
pub mod drm;

#[cfg(feature = "vulkan")]
pub mod vulkan;

pub use compositor::AureaCompositor;
pub use gpu::{GpuAvailability, GpuDevice, GpuManager, GpuManagerError};
pub use renderer::{RenderBackend, RenderPreference};
pub use server::WaylandServer;
