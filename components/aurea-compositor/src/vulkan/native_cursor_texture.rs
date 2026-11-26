// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Xcursor theme decoding and persistent Vulkan textures for named cursors.

use std::{collections::HashMap, error::Error, fmt, fs, path::PathBuf, sync::Arc};

use crate::{
    desktop::DesktopTheme,
    render::{cursor::{DEFAULT_CURSOR_HOTSPOT, default_cursor_snapshot}, shm_buffer::ShmBufferSnapshot},
};

use super::{
    VulkanLogicalDevice, VulkanShmTexture, VulkanShmTextureError,
    xcursor_file::{XcursorFileError, XcursorFrame, decode_xcursor},
};

const MAX_CURSOR_FILE_BYTES: u64 = 16 * 1024 * 1024;

/// Errors produced while loading or uploading a named cursor.
#[derive(Debug)]
pub(crate) enum VulkanNativeCursorTextureError {
    Snapshot(String),
    EmptyUpload,
    MissingAsset(String),
    File(String),
    Decode(XcursorFileError),
    Texture(VulkanShmTextureError),
}

impl fmt::Display for VulkanNativeCursorTextureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Snapshot(error) => write!(formatter, "Native cursor snapshot: {error}"),
            Self::EmptyUpload => formatter.write_str("Native cursor upload completed without a texture"),
            Self::MissingAsset(name) => write!(formatter, "Xcursor asset '{name}' was not found"),
            Self::File(error) => write!(formatter, "Xcursor file: {error}"),
            Self::Decode(error) => write!(formatter, "Xcursor decode: {error}"),
            Self::Texture(error) => write!(formatter, "Native cursor texture: {error}"),
        }
    }
}

impl Error for VulkanNativeCursorTextureError {}

/// CPU metadata and GPU images for all frames of one selected cursor size.
pub(crate) struct VulkanNativeCursorAsset {
    frames: Vec<Arc<VulkanShmTexture>>,
    width: u32,
    height: u32,
    hotspot_x: u32,
    hotspot_y: u32,
    delay_ms: u32,
}

impl VulkanNativeCursorAsset {
    pub(crate) fn texture(&self) -> Option<&VulkanShmTexture> {
        self.frames.first().map(|texture| texture.as_ref())
    }

    pub(crate) fn width(&self) -> u32 { self.width }
    pub(crate) fn height(&self) -> u32 { self.height }
    pub(crate) fn hotspot_x(&self) -> u32 { self.hotspot_x }
    pub(crate) fn hotspot_y(&self) -> u32 { self.hotspot_y }
    #[allow(dead_code)] // Consumed by the cursor animation scheduler added in the next block.
    pub(crate) fn delay_ms(&self) -> u32 { self.delay_ms }
    #[allow(dead_code)] // Retained frames are uploaded together for animated cursors.
    pub(crate) fn frame_count(&self) -> usize { self.frames.len() }
}

/// Owns the compositor's cursor textures for the lifetime of the Vulkan device.
///
/// Xcursor files are loaded once per (theme, name, requested size) and retained
/// until device teardown. Missing packaged themes fall back to the built-in arrow.
pub(crate) struct VulkanNativeCursorTexture {
    assets: HashMap<String, VulkanNativeCursorAsset>,
}

impl VulkanNativeCursorTexture {
    pub(crate) fn new() -> Self {
        Self { assets: HashMap::new() }
    }

    fn cache_key(name: &str, theme: DesktopTheme, requested_size: u32) -> String {
        let theme_name = match theme {
            DesktopTheme::Light => "AureaOS-Light",
            DesktopTheme::Dark => "AureaOS-Dark",
        };
        format!("{theme_name}/{name}/{requested_size}")
    }

    /// Loads and uploads the closest matching Xcursor image size for one output.
    pub(crate) fn ensure(
        &mut self,
        device: Arc<VulkanLogicalDevice>,
        name: &str,
        theme: DesktopTheme,
        requested_size: u32,
    ) -> Result<bool, VulkanNativeCursorTextureError> {
        let key = Self::cache_key(name, theme, requested_size);
        if self.assets.contains_key(&key) {
            return Ok(true);
        }

        // A missing theme package or cursor alias must never make the pointer disappear.
        // Keep the built-in high-contrast arrow as a deterministic fallback.
        let frames = load_cursor_frames(name, theme).or_else(|_| built_in_arrow_frame(requested_size))?;
        let selected_size = frames
            .iter()
            .map(|frame| frame.width.max(frame.height))
            .min_by_key(|size| size.abs_diff(requested_size))
            .ok_or_else(|| VulkanNativeCursorTextureError::MissingAsset(name.to_owned()))?;
        let selected = frames
            .into_iter()
            .filter(|frame| frame.width.max(frame.height) == selected_size)
            .collect::<Vec<_>>();
        let first = selected.first().ok_or_else(|| {
            VulkanNativeCursorTextureError::MissingAsset(name.to_owned())
        })?;

        let width = first.width;
        let height = first.height;
        let hotspot_x = first.hotspot_x;
        let hotspot_y = first.hotspot_y;
        let delay_ms = first.delay_ms;
        let snapshots = selected
            .into_iter()
            .map(frame_to_snapshot)
            .collect::<Result<Vec<_>, _>>()?;
        let upload = VulkanShmTexture::submit_snapshots(device, snapshots)
            .map_err(VulkanNativeCursorTextureError::Texture)?;
        let textures = upload.wait().map_err(|error| {
            VulkanNativeCursorTextureError::Texture(VulkanShmTextureError::Command(error))
        })?;
        if textures.is_empty() {
            return Err(VulkanNativeCursorTextureError::EmptyUpload);
        }

        self.assets.insert(key, VulkanNativeCursorAsset {
            frames: textures,
            width,
            height,
            hotspot_x,
            hotspot_y,
            delay_ms,
        });
        Ok(true)
    }

    pub(crate) fn asset(
        &self,
        name: &str,
        theme: DesktopTheme,
        requested_size: u32,
    ) -> Option<&VulkanNativeCursorAsset> {
        self.assets.get(&Self::cache_key(name, theme, requested_size))
    }
}

fn built_in_arrow_frame(
    requested_size: u32,
) -> Result<Vec<XcursorFrame>, VulkanNativeCursorTextureError> {
    let snapshot = default_cursor_snapshot()
        .map_err(|error| VulkanNativeCursorTextureError::Snapshot(error.to_string()))?;
    let source_size = snapshot.width();
    let size = requested_size.clamp(16, 256);
    let mut rgba8 = vec![0_u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let source_x = x * source_size / size;
            let source_y = y * source_size / size;
            let source_offset = ((source_y * source_size + source_x) * 4) as usize;
            let destination_offset = ((y * size + x) * 4) as usize;
            rgba8[destination_offset..destination_offset + 4]
                .copy_from_slice(&snapshot.pixels()[source_offset..source_offset + 4]);
        }
    }
    Ok(vec![XcursorFrame {
        width: size,
        height: size,
        hotspot_x: (DEFAULT_CURSOR_HOTSPOT.0 as u32 * size / source_size).min(size - 1),
        hotspot_y: (DEFAULT_CURSOR_HOTSPOT.1 as u32 * size / source_size).min(size - 1),
        delay_ms: 0,
        rgba8,
    }])
}

fn frame_to_snapshot(
    frame: XcursorFrame,
) -> Result<ShmBufferSnapshot, VulkanNativeCursorTextureError> {
    ShmBufferSnapshot::from_rgba8(frame.width, frame.height, frame.rgba8)
        .map_err(|error| VulkanNativeCursorTextureError::Snapshot(error.to_string()))
}

fn load_cursor_frames(
    name: &str,
    theme: DesktopTheme,
) -> Result<Vec<XcursorFrame>, VulkanNativeCursorTextureError> {
    // The root contains theme directories, e.g. /usr/share/icons/AureaOS-Light.
    // Development packages can point this at cursors/generated.
    let root = std::env::var_os("AUREA_CURSOR_THEME_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/share/icons"));
    let theme_name = match theme {
        DesktopTheme::Light => "AureaOS-Light",
        DesktopTheme::Dark => "AureaOS-Dark",
    };
    let path = root.join(theme_name).join("cursors").join(name);
    let metadata = fs::metadata(&path)
        .map_err(|error| VulkanNativeCursorTextureError::File(format!("{}: {error}", path.display())))?;
    if !metadata.is_file() || metadata.len() > MAX_CURSOR_FILE_BYTES {
        return Err(VulkanNativeCursorTextureError::File(format!(
            "{} is not a regular Xcursor file within the 16 MiB limit",
            path.display()
        )));
    }
    let bytes = fs::read(&path)
        .map_err(|error| VulkanNativeCursorTextureError::File(format!("{}: {error}", path.display())))?;
    decode_xcursor(&bytes).map_err(VulkanNativeCursorTextureError::Decode)
}
