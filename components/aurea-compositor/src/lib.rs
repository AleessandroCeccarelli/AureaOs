
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Wayland compositor component for AureaOS.
//!
//! This crate contains the compositor implementation built on Smithay.

pub mod compositor;
pub mod server;
mod xdg_shell;

pub use compositor::AureaCompositor;
pub use server::WaylandServer;
