// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli
//! Wayland compositor component for AureaOS.
pub mod compositor;
pub mod server;
mod window_manager;
mod xdg_shell;
pub use compositor::AureaCompositor;
pub use server::WaylandServer;
pub mod render;
pub mod renderer;
pub use renderer::RenderBackend;
