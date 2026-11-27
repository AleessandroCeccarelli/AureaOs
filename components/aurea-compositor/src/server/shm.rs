// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Wayland shared-memory protocol integration.

use smithay::{
    delegate_dmabuf, delegate_shm,
    backend::allocator::dmabuf::Dmabuf,
    reexports::wayland_server::protocol::wl_buffer::WlBuffer,
    wayland::{
        buffer::BufferHandler,
        dmabuf::{DmabufGlobal, DmabufHandler, DmabufState, ImportNotifier},
        shm::{ShmHandler, ShmState},
    },
};

use crate::AureaCompositor;

impl BufferHandler for AureaCompositor {
    fn buffer_destroyed(&mut self, buffer: &WlBuffer) {
        // SHM snapshots own copied pixels. DMA-BUF snapshots retain the
        // original resource until their GPU work has completed.
        #[cfg(feature = "drm")]
        self.dmabuf_buffer_destroyed(buffer);
    }
}

impl ShmHandler for AureaCompositor {
    fn shm_state(&self) -> &ShmState {
        self.wayland_shm_state()
    }
}

delegate_shm!(AureaCompositor);

#[cfg(feature = "drm")]
impl DmabufHandler for AureaCompositor {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        self.wayland_dmabuf_state()
    }

    fn dmabuf_imported(
        &mut self,
        _global: &DmabufGlobal,
        dmabuf: Dmabuf,
        notifier: ImportNotifier,
    ) {
        let Ok(layout) = crate::vulkan::DmabufImportLayout::from_dmabuf(&dmabuf) else {
            notifier.failed();
            return;
        };

        // An advertised buffer must be importable by every active rendering
        // device, not merely by whichever GPU happened to receive the request.
        // Import is performed once here to reject unsupported external memory
        // before the client receives a wl_buffer; frame rendering imports its
        // own retained image resources later.
        let devices = self.dmabuf_devices().to_vec();
        if devices.is_empty() {
            notifier.failed();
            return;
        }
        for device in devices {
            if crate::vulkan::VulkanDmabufImage::import(device, layout.clone()).is_err() {
                notifier.failed();
                return;
            }
        }

        let _ = notifier.successful::<Self>();
    }
}

#[cfg(feature = "drm")]
delegate_dmabuf!(AureaCompositor);
