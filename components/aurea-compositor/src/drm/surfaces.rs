// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Owns native Smithay DRM surfaces for discovered physical outputs.

use std::{
    collections::HashMap,
    error::Error,
    fmt,
};

use smithay::{
    backend::drm::{DrmNode, DrmSurface},
    reexports::drm::control::{connector, crtc},
};

use crate::{
    drm::{
        cpu_presentation_backend::{CpuPresentationBackend, CpuPresentationError},
        gbm_framebuffer_pool::{GbmCpuFramebufferPool, GbmCpuFramebufferPoolError},
        presentation_backend::{DrmPresentationBackend, DrmPresentationBackendError},
    },
    output::OutputId,
    vulkan::{VulkanDrmFramePool, VulkanDrmFramePoolError},
};

mod initialize;

/// Failures encountered while assigning CRTCs and creating DRM surfaces.
#[derive(Debug)]
pub enum DrmSurfaceManagerError {
    Discovery(String),
    MissingNativeConfiguration(OutputId),
    ConnectorQuery {
        output_id: OutputId,
        message: String,
    },
    EncoderQuery {
        output_id: OutputId,
        encoder_id: u32,
        message: String,
    },
    NoCompatibleCrtc(OutputId),
    DuplicateCrtc(crtc::Handle),
    SurfaceCreation {
        output_id: OutputId,
        message: String,
    },
}

impl fmt::Display for DrmSurfaceManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Discovery(message) => write!(f, "DRM output discovery failed: {message}"),
            Self::MissingNativeConfiguration(id) => write!(
                f,
                "DRM output {id} has no native connector/mode configuration"
            ),
            Self::ConnectorQuery { output_id, message } => write!(
                f,
                "Failed to query connector for output {output_id}: {message}"
            ),
            Self::EncoderQuery {
                output_id,
                encoder_id,
                message,
            } => write!(
                f,
                "Failed to query encoder {encoder_id} for output {output_id}: {message}"
            ),
            Self::NoCompatibleCrtc(id) => {
                write!(f, "No unused compatible CRTC is available for output {id}")
            }
            Self::DuplicateCrtc(crtc) => write!(f, "CRTC {crtc:?} was assigned more than once"),
            Self::SurfaceCreation { output_id, message } => write!(
                f,
                "Failed to create DRM surface for output {output_id}: {message}"
            ),
        }
    }
}

impl Error for DrmSurfaceManagerError {}

/// A physical output and its native KMS surface.
pub struct ManagedDrmSurface {
    output_id: OutputId,
    connector: connector::Handle,
    surface: DrmSurface,
    presentation: DrmPresentationBackend,
    frame_pool: VulkanDrmFramePool,
    cpu_frame_pool: GbmCpuFramebufferPool,
    cpu_presentation: CpuPresentationBackend,
}

impl ManagedDrmSurface {
    pub fn output_id(&self) -> OutputId {
        self.output_id
    }
    pub fn connector(&self) -> connector::Handle {
        self.connector
    }
    pub fn crtc(&self) -> crtc::Handle {
        self.surface.crtc()
    }
    pub fn surface(&self) -> &DrmSurface {
        &self.surface
    }
    pub fn presentation(&self) -> &DrmPresentationBackend {
        &self.presentation
    }
    pub fn presentation_mut(&mut self) -> &mut DrmPresentationBackend {
        &mut self.presentation
    }
    pub fn frame_pool(&self) -> &VulkanDrmFramePool {
        &self.frame_pool
    }
    pub fn frame_pool_mut(&mut self) -> &mut VulkanDrmFramePool {
        &mut self.frame_pool
    }
    /// Returns whether a CPU-uploaded framebuffer is awaiting its KMS page flip.
    pub(crate) fn cpu_presentation_pending(&self) -> bool {
        self.cpu_frame_pool.pending().is_some()
    }
    /// Borrows the native surface and CPU-upload presentation resources.
    pub(crate) fn cpu_parts_mut(
        &mut self,
    ) -> (
        &DrmSurface,
        &mut GbmCpuFramebufferPool,
        &mut CpuPresentationBackend,
    ) {
        (
            &self.surface,
            &mut self.cpu_frame_pool,
            &mut self.cpu_presentation,
        )
    }
    /// Borrows the KMS surface and its mutable scanout ownership resources.
    pub fn scanout_parts_mut(
        &mut self,
    ) -> (
        &DrmSurface,
        &mut VulkanDrmFramePool,
        &mut DrmPresentationBackend,
    ) {
        (&self.surface, &mut self.frame_pool, &mut self.presentation)
    }
}

/// Owns all KMS surfaces created for one DRM device.
///
/// CRTC allocation is exclusive within this manager. A manager must be
/// discarded before its parent DRM device is removed or reset.
#[derive(Default)]
pub struct DrmSurfaceManager {
    surfaces: HashMap<OutputId, ManagedDrmSurface>,
    crtc_owners: HashMap<crtc::Handle, OutputId>,
}

/// Preserves already-created KMS surfaces when a later output fails initialization.
///
/// The caller must retain partial_manager until its surfaces have been
/// successfully disabled or the DRM device is confirmed removed.
pub struct DrmSurfaceInitFailure {
    pub partial_manager: DrmSurfaceManager,
    pub cause: DrmSurfaceManagerError,
}

impl DrmSurfaceInitFailure {
    pub fn into_parts(self) -> (DrmSurfaceManager, DrmSurfaceManagerError) {
        (self.partial_manager, self.cause)
    }
}

impl DrmSurfaceManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, output_id: OutputId) -> Option<&ManagedDrmSurface> {
        self.surfaces.get(&output_id)
    }

    pub fn get_mut(&mut self, output_id: OutputId) -> Option<&mut ManagedDrmSurface> {
        self.surfaces.get_mut(&output_id)
    }

    /// Completes the Vulkan acquire and retires a framebuffer released by KMS.
    ///
    /// `None` means the CRTC no longer belongs to a managed output.
    pub fn retire_released_frame(
        &mut self,
        crtc: crtc::Handle,
        framebuffer: crate::drm::presentation::DrmFramebufferId,
    ) -> Option<Result<(), VulkanDrmFramePoolError>> {
        let output_id = self.crtc_owners.get(&crtc).copied()?;
        let output = self.surfaces.get_mut(&output_id)?;
        let (_, frame_pool, presentation) = output.scanout_parts_mut();

        Some((|| {
            let coordinator = presentation.frame_coordinator_mut();
            frame_pool.reacquire_and_recycle(framebuffer, coordinator)?;
            Ok(())
        })())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&OutputId, &ManagedDrmSurface)> {
        self.surfaces.iter()
    }

    pub fn crtc_owner(&self, crtc: crtc::Handle) -> Option<OutputId> {
        self.crtc_owners.get(&crtc).copied()
    }

    /// Completes a CPU-upload page flip for its owning output.
    pub(crate) fn on_cpu_vblank(
        &mut self,
        node: DrmNode,
        crtc: crtc::Handle,
    ) -> Result<Option<crate::drm::presentation::DrmFramebufferId>, CpuPresentationError> {
        let Some(output_id) = self.crtc_owners.get(&crtc).copied() else {
            return Ok(None);
        };
        let Some(output) = self.surfaces.get_mut(&output_id) else {
            return Ok(None);
        };
        output
            .cpu_presentation
            .on_vblank(node, crtc, &mut output.cpu_frame_pool)
    }

    /// Retires a CPU framebuffer after its page flip released KMS ownership.
    pub(crate) fn retire_released_cpu_frame(
        &mut self,
        crtc: crtc::Handle,
        framebuffer: crate::drm::presentation::DrmFramebufferId,
    ) -> Option<Result<(), GbmCpuFramebufferPoolError>> {
        let output_id = self.crtc_owners.get(&crtc).copied()?;
        let output = self.surfaces.get_mut(&output_id)?;
        Some(output.cpu_frame_pool.retire(framebuffer))
    }

    /// Routes a VBlank event to the presentation lifecycle for its CRTC.
    pub fn on_vblank(
        &mut self,
        node: DrmNode,
        crtc: crtc::Handle,
    ) -> Result<Option<crate::drm::presentation::DrmFramebufferId>, DrmPresentationBackendError>
    {
        let Some(output_id) = self.crtc_owners.get(&crtc).copied() else {
            return Ok(None);
        };
        let Some(output) = self.surfaces.get_mut(&output_id) else {
            return Ok(None);
        };
        if output.presentation().pending().is_none() {
            return Ok(None);
        }
        output.presentation_mut().on_vblank(node, crtc)
    }

    /// Releases userspace references after Udev confirms physical device removal.
    ///
    /// This must only be used from a confirmed `UdevEvent::Removed` path. The
    /// kernel has already detached the DRM device and torn down its KMS state,
    /// so issuing further disable ioctls is both unnecessary and likely to fail.
    /// Dependent GBM/Vulkan/DRM owners must still be released after this manager.
    pub fn release_after_device_removal(&mut self) {
        self.surfaces.clear();
        self.crtc_owners.clear();
    }

    /// Disables every KMS surface before releasing its buffers and ownership.
    ///
    /// Every surface is attempted. Successfully disabled surfaces are removed;
    /// failed surfaces and their CRTC ownership remain retained so callers can
    /// retry teardown and must not release dependent GBM/Vulkan resources.
    pub fn clear(&mut self) -> Result<(), String> {
        let errors = clear_and_retain(&mut self.surfaces, |managed| {
            managed.surface.clear().map_err(|error| error.to_string())
        });

        self.crtc_owners
            .retain(|_, output_id| self.surfaces.contains_key(output_id));

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }
}

impl Drop for DrmSurfaceManager {
    fn drop(&mut self) {
        // Best-effort fallback for hot-unplug and partial startup teardown.
        // Explicit pause/shutdown paths call clear() while they can report errors.
        let _ = self.clear();
    }
}

/// Runs a cleanup operation for every item and collects all failures.
#[cfg(test)]
fn clear_each<T, E>(
    items: impl IntoIterator<Item = T>,
    mut clear: impl FnMut(T) -> Result<(), E>,
) -> Vec<String>
where
    E: fmt::Display,
{
    items
        .into_iter()
        .filter_map(|item| clear(item).err().map(|error| error.to_string()))
        .collect()
}

/// Clears map values and retains entries whose cleanup failed.
fn clear_and_retain<K, V, E>(
    entries: &mut HashMap<K, V>,
    mut clear: impl FnMut(&V) -> Result<(), E>,
) -> Vec<String>
where
    K: Copy + Eq + std::hash::Hash,
    E: fmt::Display,
{
    let mut cleared = Vec::new();
    let mut errors = Vec::new();

    for (key, value) in entries.iter() {
        match clear(value) {
            Ok(()) => cleared.push(*key),
            Err(error) => errors.push(error.to_string()),
        }
    }

    for key in cleared {
        entries.remove(&key);
    }

    errors
}

#[cfg(test)]
mod lifecycle_tests {
    use super::{clear_and_retain, clear_each};
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[test]
    fn clear_attempts_every_surface_in_iteration_order() {
        let calls = RefCell::new(Vec::new());
        let errors = clear_each([1, 2, 3], |item| {
            calls.borrow_mut().push(item);
            Ok::<(), &'static str>(())
        });

        assert!(errors.is_empty());
        assert_eq!(*calls.borrow(), vec![1, 2, 3]);
    }

    #[test]
    fn clear_continues_after_failures_and_collects_them() {
        let calls = RefCell::new(Vec::new());
        let errors = clear_each([1, 2, 3], |item| {
            calls.borrow_mut().push(item);
            if item != 2 {
                Err(format!("surface {item} failed"))
            } else {
                Ok(())
            }
        });

        assert_eq!(*calls.borrow(), vec![1, 2, 3]);
        assert_eq!(errors, vec!["surface 1 failed", "surface 3 failed"]);
    }

    #[test]
    fn failed_cleanup_entry_is_retained_and_can_be_retried() {
        let mut entries = HashMap::from([(1_u8, false), (2_u8, true)]);
        let errors = clear_and_retain(&mut entries, |should_fail| {
            if *should_fail {
                Err("KMS disable failed")
            } else {
                Ok(())
            }
        });

        assert_eq!(entries.len(), 1);
        assert_eq!(entries.get(&2), Some(&true));
        assert_eq!(errors, vec!["KMS disable failed"]);

        let retry_errors = clear_and_retain(&mut entries, |_| Ok::<(), &'static str>(()));
        assert!(retry_errors.is_empty());
        assert!(entries.is_empty());
    }

    #[test]
    fn clearing_an_empty_collection_is_successful() {
        let errors = clear_each(Vec::<u8>::new(), |_| Ok::<(), &'static str>(()));
        assert!(errors.is_empty());
    }
}
