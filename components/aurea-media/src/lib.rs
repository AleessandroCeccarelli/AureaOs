// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Backend-independent image decoding contracts for AureaOS.
//! Native codec integration is private; this crate owns no GPU resources.

mod decoder;
mod error;
mod heif;
mod limits;
mod jpeg;
mod raster;
mod worker;

pub use decoder::{DecodedImage, ImageFormat, PixelFormat, decode};
pub use error::DecodeError;
pub use heif::detect_container_format as detect_heif_format;
pub use limits::{DecodeLimits, DEFAULT_DECODE_LIMITS};
pub use worker::{DecodeWorker, DecodeWorkerError, DECODE_QUEUE_CAPACITY};
