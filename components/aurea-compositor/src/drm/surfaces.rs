// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Owns native Smithay DRM surfaces for discovered physical outputs.

use std::{collections::HashMap, error::Error, fmt};

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
    UnsupportedMode { output_id: OutputId },
    AmbiguousNativeMode { output_id: OutputId },
    PendingModeChange { output_id: OutputId },
    DuplicateBatchOutput(OutputId),
    BatchRollback { output_id: OutputId, message: String },
    ModeChange { output_id: OutputId, message: String },
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
            Self::UnsupportedMode { output_id } => write!(f, "DRM output {output_id} does not support the requested logical mode"),
            Self::AmbiguousNativeMode { output_id } => write!(f, "DRM output {output_id} has multiple native timings for the requested logical mode; selection is ambiguous"),
            Self::PendingModeChange { output_id } => write!(f, "DRM output {output_id} already has a staged or pending mode change"),
            Self::DuplicateBatchOutput(id) => write!(f, "DRM output {id} appears more than once in a mode-change batch"),
            Self::BatchRollback { output_id, message } => write!(f, "DRM output {output_id} could not be restored after a batch mode-change failure: {message}"),
            Self::ModeChange { output_id, message } => write!(f, "Failed to stage DRM mode for output {output_id}: {message}"),
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
    native_modes: Vec<(crate::drm::output::DrmModeInfo, smithay::reexports::drm::control::Mode)>,
    surface: DrmSurface,
    presentation: DrmPresentationBackend,
    frame_pool: VulkanDrmFramePool,
    cpu_frame_pool: GbmCpuFramebufferPool,
    cpu_presentation: CpuPresentationBackend,
    direct_scanout_disabled: bool,
    staged_previous_mode: Option<smithay::reexports::drm::control::Mode>,
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
    /// Returns the retained native DRM timings for this connector.
    pub fn native_modes(&self) -> &[(crate::drm::output::DrmModeInfo, smithay::reexports::drm::control::Mode)] {
        &self.native_modes
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

    /// Stages the unique native timing matching a requested logical output mode.
    /// The frame renderer must allocate a framebuffer at this mode's dimensions
    /// and submit it before the staged KMS change can take effect.
    pub(crate) fn stage_native_mode(
        &mut self,
        mode: crate::output::OutputMode,
    ) -> Result<(), DrmSurfaceManagerError> {
        let mut matches = self.native_modes.iter().filter(|(info, _)| {
            info.to_output_mode() == Some(mode)
        });
        let native = matches.next().map(|(_, native)| *native).ok_or(
            DrmSurfaceManagerError::UnsupportedMode { output_id: self.output_id }
        )?;
        if matches.next().is_some() {
            return Err(DrmSurfaceManagerError::AmbiguousNativeMode { output_id: self.output_id });
        }
        let current = self.surface.current_mode();
        if current == native {
            return Ok(());
        }
        if self.staged_previous_mode.is_some()
            || self.presentation.pending().is_some()
            || self.cpu_frame_pool.pending().is_some()
        {
            return Err(DrmSurfaceManagerError::PendingModeChange {
                output_id: self.output_id,
            });
        }
        let previous = current;
        self.surface.use_mode(native).map_err(|error| DrmSurfaceManagerError::ModeChange {
            output_id: self.output_id,
            message: error.to_string(),
        })?;
        self.presentation.arm_mode_rollback(previous);
        self.cpu_presentation.arm_mode_rollback(previous);
        self.staged_previous_mode = Some(previous);
        Ok(())
    }
    /// Abandons a staged mode when rendering fails before KMS submission.
    /// A submitted/pending frame must be completed through its normal VBlank path.
    pub(crate) fn abort_staged_mode(&mut self) -> Result<(), DrmSurfaceManagerError> {
        let Some(previous) = self.staged_previous_mode else {
            return Ok(());
        };
        if self.presentation.pending().is_some() || self.cpu_frame_pool.pending().is_some() {
            return Err(DrmSurfaceManagerError::ModeChange {
                output_id: self.output_id,
                message: "cannot abort a mode while a KMS page flip is pending".into(),
            });
        }
        self.surface.use_mode(previous).map_err(|error| DrmSurfaceManagerError::ModeChange {
            output_id: self.output_id,
            message: format!("failed to restore abandoned mode: {error}"),
        })?;
        self.presentation.cancel_mode_rollback();
        self.cpu_presentation.cancel_mode_rollback();
        self.staged_previous_mode = None;
        Ok(())
    }

    /// Marks a staged mode as physically active after a completed VBlank.
    fn confirm_staged_mode(&mut self) {
        self.staged_previous_mode = None;
    }

    /// Permanently disables direct Vulkan scanout for this output after a
    /// connector-specific allocation failure; compatibility presentation remains available.
    pub(crate) fn disable_direct_scanout(&mut self) {
        self.direct_scanout_disabled = true;
    }

    /// Returns whether direct Vulkan scanout has been disabled for this output.
    pub(crate) fn direct_scanout_disabled(&self) -> bool {
        self.direct_scanout_disabled
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

    /// Stages a supported logical mode on the native KMS surface.
    ///
    /// The caller must render a framebuffer matching `mode` and submit it
    /// through the normal presentation backend. That backend performs atomic
    /// TEST_ONLY and commits the pending modeset with the new framebuffer.
    /// The previous native mode is returned so the caller can explicitly
    /// restore it if rendering or presentation is abandoned.
    pub fn stage_mode(
        &mut self,
        output_id: OutputId,
        mode: crate::output::OutputMode,
    ) -> Result<smithay::reexports::drm::control::Mode, DrmSurfaceManagerError> {
        let managed = self.surfaces.get_mut(&output_id).ok_or(
            DrmSurfaceManagerError::UnsupportedMode { output_id }
        )?;
        let mut matches = managed.native_modes.iter().filter(|(info, _)| {
            info.to_output_mode() == Some(mode)
        });
        let native = matches.next().map(|(_, native)| *native).ok_or(
            DrmSurfaceManagerError::UnsupportedMode { output_id }
        )?;
        if matches.next().is_some() {
            return Err(DrmSurfaceManagerError::AmbiguousNativeMode { output_id });
        }
        let current = managed.surface.current_mode();
        if current == native {
            return Ok(current);
        }
        if managed.staged_previous_mode.is_some()
            || managed.presentation.pending().is_some()
            || managed.cpu_frame_pool.pending().is_some()
        {
            return Err(DrmSurfaceManagerError::PendingModeChange { output_id });
        }
        let previous = current;
        managed.surface.use_mode(native).map_err(|error| {
            DrmSurfaceManagerError::ModeChange {
                output_id,
                message: error.to_string(),
            }
        })?;
        managed.presentation.arm_mode_rollback(previous);
        managed.cpu_presentation.arm_mode_rollback(previous);
        managed.staged_previous_mode = Some(previous);
        Ok(previous)
    }

    /// Stages a set of native modes as one preflight operation.
    ///
    /// No frame is submitted here. If any output cannot be staged, every
    /// earlier output in the batch is restored before the error is returned.
    /// The returned previous modes are retained by the caller for the later
    /// coordinated KMS/VBlank transaction.
    pub fn stage_modes_batch(
        &mut self,
        changes: &[(OutputId, crate::output::OutputMode)],
    ) -> Result<Vec<(OutputId, smithay::reexports::drm::control::Mode)>, DrmSurfaceManagerError> {
        let mut seen = std::collections::HashSet::with_capacity(changes.len());
        for (output_id, _) in changes {
            if !seen.insert(*output_id) {
                return Err(DrmSurfaceManagerError::DuplicateBatchOutput(*output_id));
            }
        }

        let mut staged = Vec::with_capacity(changes.len());
        for (output_id, mode) in changes {
            match self.stage_mode(*output_id, *mode) {
                Ok(previous) => staged.push((*output_id, previous)),
                Err(error) => {
                    for (staged_id, _) in staged.iter().rev() {
                        if let Err(rollback_error) = self
                            .surfaces
                            .get_mut(staged_id)
                            .ok_or(DrmSurfaceManagerError::UnsupportedMode {
                                output_id: *staged_id,
                            })
                            .and_then(ManagedDrmSurface::abort_staged_mode)
                        {
                            return Err(DrmSurfaceManagerError::BatchRollback {
                                output_id: *staged_id,
                                message: format!("{error}; rollback failed: {rollback_error}"),
                            });
                        }
                    }
                    return Err(error);
                }
            }
        }

        Ok(staged)
    }

    /// Restores a previously active native mode after an abandoned change.
    ///
    /// Restoration is staged and takes effect only when the caller submits a
    /// framebuffer through the normal TEST_ONLY/commit presentation path.
    pub fn restore_mode(
        &mut self,
        output_id: OutputId,
        previous: smithay::reexports::drm::control::Mode,
    ) -> Result<(), DrmSurfaceManagerError> {
        let managed = self.surfaces.get_mut(&output_id).ok_or(
            DrmSurfaceManagerError::UnsupportedMode { output_id }
        )?;
        managed.surface.use_mode(previous).map_err(|error| {
            DrmSurfaceManagerError::ModeChange {
                output_id,
                message: error.to_string(),
            }
        })?;
        managed.presentation.cancel_mode_rollback();
        managed.cpu_presentation.cancel_mode_rollback();
        managed.staged_previous_mode = None;
        Ok(())
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
        let completed = output
            .cpu_presentation
            .on_vblank(node, crtc, &mut output.cpu_frame_pool)?;
        if completed.is_some() {
            output.presentation.cancel_mode_rollback();
            output.confirm_staged_mode();
        }
        Ok(completed)
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
        let completed = output.presentation_mut().on_vblank(node, crtc)?;
        if completed.is_some() {
            output.cpu_presentation.cancel_mode_rollback();
            output.confirm_staged_mode();
        }
        Ok(completed)
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
        let output_ids: Vec<_> = self.surfaces.keys().copied().collect();
        let mut errors = Vec::new();
        let mut cleared = Vec::new();

        for output_id in output_ids {
            let Some(managed) = self.surfaces.get_mut(&output_id) else {
                continue;
            };

            if let Err(error) = managed.surface.clear() {
                errors.push(format!("output {output_id}: {}", error));
                continue;
            }

            // CPU-upload buffers have no Vulkan ownership transfer, but
            // their independent KMS lifecycle must still have no pending flip
            // before GBM allocations and framebuffer registrations are dropped.
            if let Err(error) = managed.cpu_frame_pool.output_disabled() {
                errors.push(format!("output {output_id} CPU framebuffer pool: {error}"));
                continue;
            }

            // The successful clear ioctl proves that KMS no longer scans
            // out the current frame. Convert that external release into an
            // explicit lifecycle transition, then wait for Vulkan to reacquire
            // foreign-owned images before releasing their backing allocations.
            let retirement = (|| {
                let coordinator = managed.presentation.frame_coordinator_mut();
                coordinator.output_disabled()?;
                managed.frame_pool.acquire_all_released(coordinator)?;
                managed.frame_pool.retire_all(coordinator)?;
                Ok::<(), VulkanDrmFramePoolError>(())
            })();
            if let Err(error) = retirement {
                errors.push(format!("output {output_id}: {error}"));
                continue;
            }

            cleared.push(output_id);
        }

        for output_id in cleared {
            self.surfaces.remove(&output_id);
        }

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
mod cleanup_tests;
