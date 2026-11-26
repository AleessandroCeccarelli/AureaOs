// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Asynchronous Xcursor decoding and persistent Vulkan textures for named cursors.

use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt, fs,
    path::PathBuf,
    sync::{
        Arc, mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError},
    },
    thread,
    time::Instant,
};

use crate::{
    desktop::DesktopTheme,
    render::{cursor::{DEFAULT_CURSOR_HOTSPOT, default_cursor_snapshot}, shm_buffer::ShmBufferSnapshot},
};

use super::{
    VulkanLogicalDevice, VulkanPendingUpload, VulkanShmTexture, VulkanShmTextureError,
    xcursor_file::{XcursorFileError, XcursorFrame, decode_xcursor},
};

const MAX_CURSOR_FILE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_PENDING_CURSOR_DECODE_REQUESTS: usize = 4;
const MAX_CACHED_CURSOR_ASSETS: usize = 32;
const MAX_CACHED_CURSOR_BYTES: usize = 16 * 1024 * 1024;
const MAX_CURSOR_ASSET_BYTES: usize = 4 * 1024 * 1024;

/// Errors produced while loading or uploading a named cursor.
#[derive(Debug)]
pub(crate) enum VulkanNativeCursorTextureError {
    Snapshot(String),
    EmptyUpload,
    MissingAsset(String),
    File(String),
    Decode(XcursorFileError),
    Texture(VulkanShmTextureError),
    CacheBudgetExceeded,
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
            Self::CacheBudgetExceeded => formatter.write_str("Native cursor texture cache budget exceeded"),
        }
    }
}

impl Error for VulkanNativeCursorTextureError {}

pub(crate) struct VulkanNativeCursorFrame {
    texture: Arc<VulkanShmTexture>,
    width: u32,
    height: u32,
    hotspot_x: u32,
    hotspot_y: u32,
    delay_ms: u32,
}

/// CPU metadata and GPU images for every frame of one selected cursor size.
pub(crate) struct VulkanNativeCursorAsset {
    frames: Vec<VulkanNativeCursorFrame>,
    animation_started_at: Instant,
}

impl VulkanNativeCursorAsset {
    /// Returns the frame selected by the Xcursor per-frame delays.
    pub(crate) fn frame_at(&self, now: Instant) -> &VulkanNativeCursorFrame {
        if self.frames.len() <= 1 {
            return &self.frames[0];
        }

        let delays = self.frames.iter()
            .map(|frame| frame.delay_ms)
            .collect::<Vec<_>>();
        let elapsed_ms = now.saturating_duration_since(self.animation_started_at).as_millis() as u64;
        &self.frames[frame_index_for_elapsed(&delays, elapsed_ms)]
    }

    pub(crate) fn has_animation(&self) -> bool {
        self.frames.len() > 1
    }
}

impl VulkanNativeCursorFrame {
    pub(crate) fn texture(&self) -> &VulkanShmTexture { self.texture.as_ref() }
    pub(crate) fn width(&self) -> u32 { self.width }
    pub(crate) fn height(&self) -> u32 { self.height }
    pub(crate) fn hotspot_x(&self) -> u32 { self.hotspot_x }
    pub(crate) fn hotspot_y(&self) -> u32 { self.hotspot_y }
    pub(crate) fn delay_ms(&self) -> u32 { self.delay_ms }
}

fn frame_index_for_elapsed(delays_ms: &[u32], elapsed_ms: u64) -> usize {
    if delays_ms.len() <= 1 {
        return 0;
    }
    let total_ms = delays_ms.iter()
        .map(|delay| u64::from((*delay).max(1)))
        .sum::<u64>()
        .max(1);
    let mut position_ms = elapsed_ms % total_ms;
    for (index, delay) in delays_ms.iter().enumerate() {
        let duration = u64::from((*delay).max(1));
        if position_ms < duration {
            return index;
        }
        position_ms -= duration;
    }
    0
}

#[cfg(test)]
mod animation_tests {
    use super::{built_in_arrow_frame, frame_index_for_elapsed};

    #[test]
    fn selects_frames_using_xcursor_delays_and_wraps() {
        let delays = [100, 200, 300];
        assert_eq!(frame_index_for_elapsed(&delays, 0), 0);
        assert_eq!(frame_index_for_elapsed(&delays, 99), 0);
        assert_eq!(frame_index_for_elapsed(&delays, 100), 1);
        assert_eq!(frame_index_for_elapsed(&delays, 299), 1);
        assert_eq!(frame_index_for_elapsed(&delays, 300), 2);
        assert_eq!(frame_index_for_elapsed(&delays, 599), 2);
        assert_eq!(frame_index_for_elapsed(&delays, 600), 0);
    }

    #[test]
    fn treats_zero_delay_as_a_minimum_one_millisecond_frame() {
        assert_eq!(frame_index_for_elapsed(&[0, 10], 0), 0);
        assert_eq!(frame_index_for_elapsed(&[0, 10], 1), 1);
    }

    #[test]
    fn built_in_fallback_has_valid_dimensions_hotspot_and_rgba_payload() {
        let frame = built_in_arrow_frame(64).unwrap().remove(0);

        assert_eq!((frame.width, frame.height), (64, 64));
        assert_eq!((frame.hotspot_x, frame.hotspot_y), (10, 6));
        assert_eq!(frame.rgba8.len(), 64 * 64 * 4);
        assert_eq!(frame.delay_ms, 0);
    }

    #[test]
    fn built_in_fallback_clamps_extreme_raster_sizes() {
        let small = built_in_arrow_frame(1).unwrap().remove(0);
        let large = built_in_arrow_frame(4096).unwrap().remove(0);

        assert_eq!((small.width, small.height), (16, 16));
        assert_eq!((large.width, large.height), (256, 256));
    }
}

struct CursorDecodeRequest {
    key: String,
    name: String,
    theme: DesktopTheme,
    requested_size: u32,
}

struct CursorDecodeResult {
    key: String,
    name: String,
    requested_size: u32,
    frames: Result<Vec<XcursorFrame>, String>,
}

struct PendingCursorUpload {
    key: String,
    upload: VulkanPendingUpload<Vec<Arc<VulkanShmTexture>>>,
    metadata: Vec<XcursorFrame>,
}

/// Owns cursor decoding and GPU resources for the lifetime of the Vulkan device.
///
/// File I/O and Xcursor parsing run on a dedicated bounded worker. Vulkan uploads
/// are fence-polled by the render thread and never waited on during normal frames.
struct CachedNativeCursorAsset {
    asset: VulkanNativeCursorAsset,
    byte_len: usize,
    last_used: u64,
}

/// Owns cursor decoding and GPU resources for the lifetime of the Vulkan device.
///
/// File I/O and Xcursor parsing run on a dedicated bounded worker. Vulkan uploads
/// are fence-polled by the render thread and never waited on during normal frames.
pub(crate) struct VulkanNativeCursorTexture {
    assets: HashMap<String, CachedNativeCursorAsset>,
    /// Small built-in arrows are uploaded once per requested raster size and
    /// remain available while the themed Xcursor asset is decoded.
    fallback_assets: HashMap<u32, VulkanNativeCursorAsset>,
    pending_keys: HashSet<String>,
    pending_uploads: Vec<PendingCursorUpload>,
    decode_tx: Option<SyncSender<CursorDecodeRequest>>,
    decode_rx: Receiver<CursorDecodeResult>,
    cached_bytes: usize,
    frame: u64,
}

impl VulkanNativeCursorTexture {
    pub(crate) fn new() -> Self {
        let (request_tx, request_rx) = mpsc::sync_channel::<CursorDecodeRequest>(
            MAX_PENDING_CURSOR_DECODE_REQUESTS,
        );
        let (result_tx, result_rx) = mpsc::sync_channel::<CursorDecodeResult>(
            MAX_PENDING_CURSOR_DECODE_REQUESTS,
        );
        let worker = thread::Builder::new()
            .name("aurea-xcursor-decode".to_owned())
            .spawn(move || {
                while let Ok(request) = request_rx.recv() {
                    let result = load_cursor_frames(&request.name, request.theme)
                        .or_else(|_| built_in_arrow_frame(request.requested_size))
                        .map_err(|error| error.to_string());
                    if result_tx.send(CursorDecodeResult {
                        key: request.key,
                        name: request.name,
                        requested_size: request.requested_size,
                        frames: result,
                    }).is_err() {
                        break;
                    }
                }
            })
            .ok();

        Self {
            assets: HashMap::new(),
            fallback_assets: HashMap::new(),
            pending_keys: HashSet::new(),
            pending_uploads: Vec::new(),
            decode_tx: worker.map(|_| request_tx),
            decode_rx: result_rx,
            cached_bytes: 0,
            frame: 0,
        }
    }

    fn cache_key(name: &str, theme: DesktopTheme, requested_size: u32) -> String {
        let theme_name = match theme {
            DesktopTheme::Light => "AureaOS-Light",
            DesktopTheme::Dark => "AureaOS-Dark",
        };
        format!("{theme_name}/{name}/{requested_size}")
    }

    /// Polls worker results and GPU fences, then queues a missing asset if needed.
    ///
    /// Returns true only after every selected frame has completed its GPU upload.
    pub(crate) fn ensure(
        &mut self,
        device: Arc<VulkanLogicalDevice>,
        name: &str,
        theme: DesktopTheme,
        requested_size: u32,
    ) -> Result<bool, VulkanNativeCursorTextureError> {
        self.frame = self.frame.wrapping_add(1);
        self.poll_uploads()?;
        self.poll_decoded_frames(Arc::clone(&device))?;

        let key = Self::cache_key(name, theme, requested_size);
        if let Some(entry) = self.assets.get_mut(&key) {
            entry.last_used = self.frame;
            return Ok(true);
        }

        // Never leave the desktop without a pointer while file I/O, Xcursor
        // decoding or the themed texture upload is still in flight. The
        // built-in 32x32-derived arrow is tiny and is uploaded only once for
        // each output raster size; the themed asset replaces it when ready.
        self.ensure_fallback_cursor(Arc::clone(&device), requested_size)?;

        if self.pending_keys.contains(&key) {
            return Ok(self.fallback_assets.contains_key(&requested_size));
        }

        let Some(sender) = &self.decode_tx else {
            return Ok(self.fallback_assets.contains_key(&requested_size));
        };
        let request = CursorDecodeRequest {
            key: key.clone(),
            name: name.to_owned(),
            theme,
            requested_size,
        };
        match sender.try_send(request) {
            Ok(()) => {
                self.pending_keys.insert(key);
            }
            Err(TrySendError::Full(_)) => {}
            Err(TrySendError::Disconnected(_)) => {
                return Ok(self.fallback_assets.contains_key(&requested_size));
            }
        }
        Ok(self.fallback_assets.contains_key(&requested_size))
    }

    fn ensure_fallback_cursor(
        &mut self,
        device: Arc<VulkanLogicalDevice>,
        requested_size: u32,
    ) -> Result<(), VulkanNativeCursorTextureError> {
        if self.fallback_assets.contains_key(&requested_size) {
            return Ok(());
        }

        let metadata = built_in_arrow_frame(requested_size)?
            .into_iter()
            .next()
            .ok_or(VulkanNativeCursorTextureError::EmptyUpload)?;
        let snapshot = ShmBufferSnapshot::from_rgba8(
            metadata.width,
            metadata.height,
            metadata.rgba8.clone(),
        )
        .map_err(|error| VulkanNativeCursorTextureError::Snapshot(error.to_string()))?;
        // This one-time upload is deliberately synchronous and extremely small:
        // the initial arrow must be drawable on the first output frame.
        let texture = Arc::new(
            VulkanShmTexture::from_snapshot(device, snapshot)
                .map_err(VulkanNativeCursorTextureError::Texture)?,
        );
        let frame = VulkanNativeCursorFrame {
            texture,
            width: metadata.width,
            height: metadata.height,
            hotspot_x: metadata.hotspot_x,
            hotspot_y: metadata.hotspot_y,
            delay_ms: 0,
        };
        // Four raster sizes cover normal and high-DPI outputs without allowing
        // fallback textures to grow without bound.
        if self.fallback_assets.len() >= 4 {
            if let Some(oldest_size) = self.fallback_assets.keys().next().copied() {
                self.fallback_assets.remove(&oldest_size);
            }
        }
        self.fallback_assets.insert(
            requested_size,
            VulkanNativeCursorAsset {
                frames: vec![frame],
                animation_started_at: Instant::now(),
            },
        );
        Ok(())
    }

    fn poll_decoded_frames(
        &mut self,
        device: Arc<VulkanLogicalDevice>,
    ) -> Result<(), VulkanNativeCursorTextureError> {
        loop {
            let result = match self.decode_rx.try_recv() {
                Ok(result) => result,
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            };
            self.pending_keys.remove(&result.key);
            let frames = result.frames.map_err(|error| {
                VulkanNativeCursorTextureError::File(format!("{}: {error}", result.name))
            })?;
            let selected_size = frames
                .iter()
                .map(|frame| frame.width.max(frame.height))
                .min_by_key(|size| size.abs_diff(result.requested_size))
                .ok_or_else(|| VulkanNativeCursorTextureError::MissingAsset(result.name.clone()))?;
            let mut metadata = frames.into_iter()
                .filter(|frame| frame.width.max(frame.height) == selected_size)
                .collect::<Vec<_>>();
            if metadata.is_empty() {
                return Err(VulkanNativeCursorTextureError::MissingAsset(result.name));
            }
            let frame_bytes = metadata.iter().try_fold(0usize, |total, frame| {
                total.checked_add(frame.rgba8.len())
            }).ok_or(VulkanNativeCursorTextureError::CacheBudgetExceeded)?;
            // Preserve the cursor's first frame rather than allowing an unusually
            // large animation to consume the entire native artwork budget.
            if frame_bytes > MAX_CURSOR_ASSET_BYTES {
                metadata.truncate(1);
            }

            let snapshots = metadata.iter()
                .map(|frame| {
                    ShmBufferSnapshot::from_rgba8(frame.width, frame.height, frame.rgba8.clone())
                        .map_err(|error| VulkanNativeCursorTextureError::Snapshot(error.to_string()))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let upload = VulkanShmTexture::submit_snapshots(Arc::clone(&device), snapshots)
                .map_err(VulkanNativeCursorTextureError::Texture)?;
            self.pending_uploads.push(PendingCursorUpload {
                key: result.key,
                upload,
                metadata,
            });
        }
        Ok(())
    }

    fn poll_uploads(&mut self) -> Result<(), VulkanNativeCursorTextureError> {
        let mut index = 0;
        while index < self.pending_uploads.len() {
            let complete = self.pending_uploads[index].upload.is_complete()
                .map_err(|error| {
                    VulkanNativeCursorTextureError::Texture(VulkanShmTextureError::Command(error))
                })?;
            if !complete {
                index += 1;
                continue;
            }

            let pending = self.pending_uploads.remove(index);
            let textures = pending.upload.wait().map_err(|error| {
                VulkanNativeCursorTextureError::Texture(VulkanShmTextureError::Command(error))
            })?;
            if textures.len() != pending.metadata.len() || textures.is_empty() {
                return Err(VulkanNativeCursorTextureError::EmptyUpload);
            }
            let byte_len = pending.metadata.iter()
                .map(|frame| frame.rgba8.len())
                .sum::<usize>();
            let frames = pending.metadata.into_iter().zip(textures).map(|(metadata, texture)| {
                VulkanNativeCursorFrame {
                    texture,
                    width: metadata.width,
                    height: metadata.height,
                    hotspot_x: metadata.hotspot_x,
                    hotspot_y: metadata.hotspot_y,
                    delay_ms: metadata.delay_ms,
                }
            }).collect();
            if byte_len > MAX_CACHED_CURSOR_BYTES {
                return Err(VulkanNativeCursorTextureError::CacheBudgetExceeded);
            }
            while self.assets.len() >= MAX_CACHED_CURSOR_ASSETS
                || self.cached_bytes.checked_add(byte_len)
                    .map_or(true, |total| total > MAX_CACHED_CURSOR_BYTES)
            {
                let oldest = self.assets.iter()
                    .min_by_key(|(_, entry)| entry.last_used)
                    .map(|(key, _)| key.clone());
                let Some(oldest) = oldest else {
                    return Err(VulkanNativeCursorTextureError::CacheBudgetExceeded);
                };
                if let Some(removed) = self.assets.remove(&oldest) {
                    self.cached_bytes = self.cached_bytes.saturating_sub(removed.byte_len);
                }
            }
            self.cached_bytes = self.cached_bytes.saturating_add(byte_len);
            self.assets.insert(pending.key, CachedNativeCursorAsset {
                asset: VulkanNativeCursorAsset {
                    frames,
                    animation_started_at: Instant::now(),
                },
                byte_len,
                last_used: self.frame,
            });
        }
        Ok(())
    }

    /// Reports decode or GPU-upload work that still needs another render poll.
    pub(crate) fn has_pending_work(&self) -> bool {
        !self.pending_keys.is_empty() || !self.pending_uploads.is_empty()
    }

    pub(crate) fn asset(
        &self,
        name: &str,
        theme: DesktopTheme,
        requested_size: u32,
    ) -> Option<&VulkanNativeCursorAsset> {
        self.assets.get(&Self::cache_key(name, theme, requested_size))
            .map(|entry| &entry.asset)
            .or_else(|| self.fallback_assets.get(&requested_size))
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
