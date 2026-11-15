// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Bounded, per-device cache of uploaded Wayland SHM surface textures.

use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt,
    sync::Arc,
};

use smithay::reexports::wayland_server::backend::ObjectId;

use crate::render::shm_buffer::ShmBufferSnapshot;

use super::{VulkanLogicalDevice, VulkanShmTexture, VulkanShmTextureError};

const MAX_CACHED_SURFACE_TEXTURES: usize = 128;
const MAX_CACHED_SURFACE_TEXTURE_BYTES: usize = 256 * 1024 * 1024;

/// Errors returned while synchronizing the SHM texture cache.
#[derive(Debug)]
pub(crate) enum VulkanSurfaceTextureCacheError {
    Texture(VulkanShmTextureError),
    CacheBudgetExceeded,
}

impl fmt::Display for VulkanSurfaceTextureCacheError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Texture(error) => write!(formatter, "SHM texture cache upload failed: {error}"),
            Self::CacheBudgetExceeded => formatter.write_str(
                "Visible SHM textures exceed the cache resource budget",
            ),
        }
    }
}

impl Error for VulkanSurfaceTextureCacheError {}

struct CachedSurfaceTexture {
    revision: u64,
    snapshot: Arc<ShmBufferSnapshot>,
    texture: VulkanShmTexture,
    byte_len: usize,
    last_used: u64,
}

/// Retains uploaded textures while their Wayland surfaces remain active.
///
/// Entries use the full Wayland ObjectId rather than its reusable protocol
/// integer. The bounded LRU policy protects textures required by the current frame.
pub(crate) struct VulkanSurfaceTextureCache {
    device: Arc<VulkanLogicalDevice>,
    entries: HashMap<ObjectId, CachedSurfaceTexture>,
    cached_bytes: usize,
    frame: u64,
}

impl VulkanSurfaceTextureCache {
    pub(crate) fn new(device: Arc<VulkanLogicalDevice>) -> Self {
        Self {
            device,
            entries: HashMap::new(),
            cached_bytes: 0,
            frame: 0,
        }
    }

    /// Removes inactive surfaces and prepares the textures needed by this output.
    pub(crate) fn prepare(
        &mut self,
        active_surface_ids: &[ObjectId],
        visible_surfaces: &[(ObjectId, u64, Arc<ShmBufferSnapshot>)],
    ) -> Result<(), VulkanSurfaceTextureCacheError> {
        self.frame = self.frame.wrapping_add(1);
        let active: HashSet<_> = active_surface_ids.iter().cloned().collect();
        self.entries.retain(|id, _| active.contains(id));
        self.recalculate_cached_bytes();

        let required: HashSet<_> = visible_surfaces
            .iter()
            .map(|(id, _, _)| id.clone())
            .collect();
        let requested_bytes = visible_surfaces
            .iter()
            .try_fold(0usize, |total, (_, _, snapshot)| {
                total.checked_add(snapshot.pixels().len())
            })
            .ok_or(VulkanSurfaceTextureCacheError::CacheBudgetExceeded)?;

        if visible_surfaces.len() > MAX_CACHED_SURFACE_TEXTURES
            || requested_bytes > MAX_CACHED_SURFACE_TEXTURE_BYTES
        {
            return Err(VulkanSurfaceTextureCacheError::CacheBudgetExceeded);
        }

        // Remove every stale revision before allocating replacements. Otherwise
        // old images for later surfaces in this same frame could be protected by
        // the required set and temporarily exhaust the budget.
        for (surface_id, revision, snapshot) in visible_surfaces {
            let stale = self.entries.get(surface_id).is_some_and(|entry| {
                entry.revision != *revision || !Arc::ptr_eq(&entry.snapshot, snapshot)
            });
            if stale {
                self.remove_surface(surface_id);
            }
        }

        let mut missing = Vec::new();
        let mut missing_bytes = 0usize;
        for (surface_id, revision, snapshot) in visible_surfaces {
            if let Some(entry) = self.entries.get_mut(surface_id)
                && entry.revision == *revision
                && Arc::ptr_eq(&entry.snapshot, snapshot)
            {
                entry.last_used = self.frame;
                continue;
            }

            missing_bytes = missing_bytes
                .checked_add(snapshot.pixels().len())
                .ok_or(VulkanSurfaceTextureCacheError::CacheBudgetExceeded)?;
            missing.push((surface_id.clone(), *revision, Arc::clone(snapshot)));
        }

        while self.entries.len().saturating_add(missing.len()) > MAX_CACHED_SURFACE_TEXTURES
            || self
                .cached_bytes
                .checked_add(missing_bytes)
                .map_or(true, |total| total > MAX_CACHED_SURFACE_TEXTURE_BYTES)
        {
            let candidate = self
                .entries
                .iter()
                .filter(|(id, _)| !required.contains(*id))
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(id, _)| id.clone());

            let Some(candidate) = candidate else {
                return Err(VulkanSurfaceTextureCacheError::CacheBudgetExceeded);
            };
            self.remove_surface(&candidate);
        }

        let snapshots = missing
            .iter()
            .map(|(_, _, snapshot)| snapshot.as_ref().clone())
            .collect();
        let textures = VulkanShmTexture::from_snapshots(Arc::clone(&self.device), snapshots)
            .map_err(VulkanSurfaceTextureCacheError::Texture)?;

        for ((surface_id, revision, snapshot), texture) in missing.into_iter().zip(textures) {
            let byte_len = snapshot.pixels().len();
            self.cached_bytes += byte_len;
            self.entries.insert(
                surface_id,
                CachedSurfaceTexture {
                    revision,
                    snapshot,
                    texture,
                    byte_len,
                    last_used: self.frame,
                },
            );
        }

        Ok(())
    }

    pub(crate) fn texture(&self, surface_id: &ObjectId) -> Option<&VulkanShmTexture> {
        self.entries.get(surface_id).map(|entry| &entry.texture)
    }

    fn remove_surface(&mut self, surface_id: &ObjectId) {
        if let Some(entry) = self.entries.remove(surface_id) {
            self.cached_bytes = self.cached_bytes.saturating_sub(entry.byte_len);
        }
    }

    fn recalculate_cached_bytes(&mut self) {
        self.cached_bytes = self.entries.values().map(|entry| entry.byte_len).sum();
    }
}
