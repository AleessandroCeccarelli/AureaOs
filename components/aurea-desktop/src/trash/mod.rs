// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Freedesktop-compatible home Trash storage.

mod metadata;
mod service;
mod timestamp;

pub use service::{TrashEntry, TrashError, TrashService, TrashSnapshot, TrashVisualState};
