// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Linux DMA-BUF producer synchronization through kernel sync files.

use std::{
    error::Error,
    fmt,
    os::fd::{AsRawFd, BorrowedFd, OwnedFd, FromRawFd},
};

use super::DmabufImportLayout;

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
