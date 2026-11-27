// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Linux DMA-BUF producer synchronization through kernel sync files.

use std::{
    error::Error,
    fmt,
    os::fd::{AsRawFd, OwnedFd, FromRawFd, IntoRawFd},
};

use smithay::reexports::ash::{self, vk};

use super::{DmabufImportLayout, VulkanLogicalDevice};

/// Linux DMA_BUF_IOCTL_EXPORT_SYNC_FILE request number.
///
/// _IOWR('b', 2, struct dma_buf_export_sync_file), whose ABI payload is
/// two 32-bit integers (flags and returned file descriptor).
const DMA_BUF_IOCTL_EXPORT_SYNC_FILE: libc::c_ulong = 0xC008_6202;
/// Wait for all prior device writes before this compositor reads the buffer.
const DMA_BUF_SYNC_READ: u32 = 1;

/// Kernel ABI for DMA_BUF_IOCTL_EXPORT_SYNC_FILE.
#[repr(C)]
#[derive(Debug, Default)]
struct DmaBufExportSyncFile {
    flags: u32,
    fd: i32,
}

/// Failure to obtain a producer fence from a client DMA-BUF.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DmabufSyncFileError {
    MissingPlane,
    ExportIo(std::io::ErrorKind),
    InvalidReturnedFd,
}

impl fmt::Display for DmabufSyncFileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPlane => formatter.write_str("DMA-BUF has no memory plane"),
            Self::ExportIo(kind) => write!(formatter, "DMA-BUF sync-file export failed: {kind:?}"),
            Self::InvalidReturnedFd => formatter.write_str("Kernel returned an invalid sync-file descriptor"),
        }
    }
}

impl Error for DmabufSyncFileError {}

/// Exports a read fence for the first DMA-BUF plane.
///
/// The returned owned descriptor must be imported into a Vulkan SYNC_FD
/// semaphore before any sampling command is submitted. This function does
/// not imply that the device supports Vulkan SYNC_FD import.
pub(crate) fn export_read_sync_file(
    layout: &DmabufImportLayout,
) -> Result<OwnedFd, DmabufSyncFileError> {
    let plane = layout
        .plane_handles()
        .next()
        .ok_or(DmabufSyncFileError::MissingPlane)?;

    let mut request = DmaBufExportSyncFile {
        flags: DMA_BUF_SYNC_READ,
        fd: -1,
    };
    // SAFETY: the ioctl ABI requires a writable pointer to the fixed-size
    // repr(C) structure; the borrowed DMA-BUF descriptor remains valid here.
    let result = unsafe {
        libc::ioctl(
            plane.as_raw_fd(),
            DMA_BUF_IOCTL_EXPORT_SYNC_FILE,
            &mut request as *mut DmaBufExportSyncFile,
        )
    };
    if result < 0 {
        return Err(DmabufSyncFileError::ExportIo(
            std::io::Error::last_os_error().kind(),
        ));
    }
    if request.fd < 0 {
        return Err(DmabufSyncFileError::InvalidReturnedFd);
    }

    // SAFETY: successful DMA_BUF_IOCTL_EXPORT_SYNC_FILE transfers a newly
    // created sync-file descriptor to the caller.
    Ok(unsafe { OwnedFd::from_raw_fd(request.fd) })
}


/// Imports a Linux sync-file as a temporary Vulkan binary-semaphore payload.
///
/// On success Vulkan owns the file descriptor and the returned semaphore must
/// be included in the queue submission wait list. The caller must destroy the
/// semaphore only after that submission has completed.
pub(crate) fn import_sync_file_semaphore(
    device: &VulkanLogicalDevice,
    sync_file: OwnedFd,
) -> Result<vk::Semaphore, vk::Result> {
    if device.is_unusable() {
        return Err(vk::Result::ERROR_DEVICE_LOST);
    }
    if !device.supports_sync_fd_import() {
        return Err(vk::Result::ERROR_FEATURE_NOT_PRESENT);
    }

    let mut export_info = vk::ExportSemaphoreCreateInfo::default()
        .handle_types(vk::ExternalSemaphoreHandleTypeFlags::SYNC_FD);
    let create_info = vk::SemaphoreCreateInfo::default().push_next(&mut export_info);
    let semaphore = unsafe { device.handle().create_semaphore(&create_info, None) }?;

    let raw_fd = sync_file.into_raw_fd();
    let import_info = vk::ImportSemaphoreFdInfoKHR::default()
        .semaphore(semaphore)
        .flags(vk::SemaphoreImportFlags::TEMPORARY)
        .handle_type(vk::ExternalSemaphoreHandleTypeFlags::SYNC_FD)
        .fd(raw_fd);
    let loader = ash::khr::external_semaphore_fd::Device::new(
        device.instance(),
        device.handle(),
    );
    if let Err(error) = unsafe { loader.import_semaphore_fd(&import_info) } {
        // The FD ownership transfer occurs only on successful import.
        unsafe {
            drop(OwnedFd::from_raw_fd(raw_fd));
            device.handle().destroy_semaphore(semaphore, None);
        }
        return Err(error);
    }

    Ok(semaphore)
}

/// Destroys an imported producer-fence semaphore after its wait submission completes.
pub(crate) fn destroy_imported_sync_file_semaphore(
    device: &VulkanLogicalDevice,
    semaphore: vk::Semaphore,
) {
    if !device.is_unusable() {
        unsafe { device.handle().destroy_semaphore(semaphore, None) };
    }
}

/// Owns imported producer-fence semaphores until the waiting queue submission completes.
pub(crate) struct DmabufSyncWaitSet {
    device: std::sync::Arc<VulkanLogicalDevice>,
    semaphores: Vec<vk::Semaphore>,
    quarantined: bool,
}

impl DmabufSyncWaitSet {
    pub(crate) fn new(device: std::sync::Arc<VulkanLogicalDevice>) -> Self {
        Self { device, semaphores: Vec::new(), quarantined: false }
    }

    pub(crate) fn import_layout(
        &mut self,
        layout: &DmabufImportLayout,
    ) -> Result<(), DmabufSyncWaitError> {
        let sync_file = export_read_sync_file(layout).map_err(DmabufSyncWaitError::Export)?;
        let semaphore = import_sync_file_semaphore(&self.device, sync_file)
            .map_err(DmabufSyncWaitError::Import)?;
        self.semaphores.push(semaphore);
        Ok(())
    }

    pub(crate) fn wait_semaphores(&self) -> &[vk::Semaphore] {
        &self.semaphores
    }

    /// Prevents destruction when queue acceptance or completion is uncertain.
    pub(crate) fn quarantine(mut self) {
        self.quarantined = true;
        std::mem::forget(self);
    }
}

impl Drop for DmabufSyncWaitSet {
    fn drop(&mut self) {
        if self.quarantined || self.device.is_unusable() {
            return;
        }
        for semaphore in self.semaphores.drain(..) {
            destroy_imported_sync_file_semaphore(&self.device, semaphore);
        }
    }
}

/// Failure while exporting or importing one DMA-BUF producer fence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DmabufSyncWaitError {
    Export(DmabufSyncFileError),
    Import(vk::Result),
}

impl fmt::Display for DmabufSyncWaitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Export(error) => write!(formatter, "{error}"),
            Self::Import(error) => write!(formatter, "Vulkan SYNC_FD import failed: {error:?}"),
        }
    }
}

impl Error for DmabufSyncWaitError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_sync_file_ioctl_uses_linux_uapi_number() {
        assert_eq!(DMA_BUF_IOCTL_EXPORT_SYNC_FILE, 0xC008_6202);
    }

    #[test]
    fn export_sync_file_payload_matches_linux_uapi_layout() {
        assert_eq!(std::mem::size_of::<DmaBufExportSyncFile>(), 8);
        assert_eq!(std::mem::align_of::<DmaBufExportSyncFile>(), 4);
    }
}
