// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

mod container;

#[cfg(feature = "heif")]
mod decoder;

pub use container::detect_container_format;

#[cfg(feature = "heif")]
pub(crate) use decoder::decode;
