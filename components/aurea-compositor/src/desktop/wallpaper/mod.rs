// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Desktop wallpaper assets, layout, decoded pixels, and animation contracts.

mod animation;
mod asset;
mod layout;
mod manager;
mod pixels;
mod snapshot;

pub use animation::{
    MAX_WALLPAPER_FRAME_RATE, MIN_WALLPAPER_FRAME_RATE, WallpaperFrame, WallpaperFrameProvider,
    WallpaperPauseReason, WallpaperPlaybackDecision, WallpaperPlaybackEnvironment,
    WallpaperPlaybackPolicy, WallpaperProviderError,
};
pub use asset::{WallpaperAsset, WallpaperDimensions, WallpaperFormat};
pub use layout::{WallpaperFitMode, WallpaperLayout, WallpaperPixelRect};
pub use manager::WallpaperManager;
pub use pixels::{MAX_WALLPAPER_RGBA_BYTES, WallpaperPixels, WallpaperPixelsError};
pub use snapshot::WallpaperSnapshot;
