// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Desktop wallpaper assets, layout, decoded pixels, and animation contracts.

mod animation;
mod asset;
mod manager;
mod pixels;
mod layout;
mod snapshot;

pub use animation::{
    WallpaperFrame, WallpaperFrameProvider, WallpaperPauseReason, WallpaperPlaybackDecision,
    WallpaperPlaybackEnvironment, WallpaperPlaybackPolicy, WallpaperProviderError,
    MAX_WALLPAPER_FRAME_RATE, MIN_WALLPAPER_FRAME_RATE,
};
pub use asset::{WallpaperAsset, WallpaperDimensions, WallpaperFormat};
pub use manager::WallpaperManager;
pub use layout::{WallpaperFitMode, WallpaperLayout, WallpaperPixelRect};
pub use pixels::{WallpaperPixels, WallpaperPixelsError, MAX_WALLPAPER_RGBA_BYTES};
pub use snapshot::WallpaperSnapshot;
