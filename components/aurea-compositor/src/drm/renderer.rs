// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! GBM-backed OpenGL ES renderer management for DRM devices.

use std::{error::Error, fmt};

use smithay::backend::{
    allocator::gbm::GbmDevice,
    drm::{DrmDeviceFd, DrmNode},
    egl::context::ContextPriority,
    renderer::{
        gles::GlesRenderer,
        multigpu::{GpuManager, MultiRenderer, gbm::GbmGlesBackend},
    },
};

/// A renderer backed by Smithay's GBM GLES graphics API.
pub type GbmGlesRenderer<'a> = MultiRenderer<
    'a,
    'a,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>,
>;

type GbmGlesApi = GbmGlesBackend<GlesRenderer, DrmDeviceFd>;

/// Errors encountered while preparing a renderer for a DRM device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GbmGlesRendererError {
    /// GBM could not create a device for the opened DRM file descriptor.
    GbmDevice(String),
    /// Smithay could not initialize or select a renderer for the DRM node.
    Renderer(String),
}

impl fmt::Display for GbmGlesRendererError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GbmDevice(message) => {
                write!(formatter, "Failed to initialize the GBM device: {message}")
            }
            Self::Renderer(message) => {
                write!(formatter, "Failed to initialize the GLES renderer: {message}")
            }
        }
    }
}

impl Error for GbmGlesRendererError {}

/// Owns Smithay's GLES renderer manager and registers opened DRM devices.
///
/// Renderer creation is lazy: registering a device prepares its GBM/EGL
/// display, while the first call to [`with_renderer`](Self::with_renderer)
/// creates the GLES context on the compositor thread that uses it.
pub struct GbmGlesRendererManager {
    manager: GpuManager<GbmGlesApi>,
}

impl GbmGlesRendererManager {
    /// Creates an empty renderer manager with medium-priority GLES contexts.
    pub fn new() -> Result<Self, GbmGlesRendererError> {
        let api = GbmGlesBackend::with_context_priority(ContextPriority::Medium);
        let manager = GpuManager::new(api)
            .map_err(|error| GbmGlesRendererError::Renderer(error.to_string()))?;

        Ok(Self { manager })
    }

    /// Registers a GBM device for an already opened DRM node.
    pub fn add_device(
        &mut self,
        node: DrmNode,
        device_fd: &DrmDeviceFd,
    ) -> Result<(), GbmGlesRendererError> {
        let gbm_device = GbmDevice::new(device_fd.clone())
            .map_err(|error| GbmGlesRendererError::GbmDevice(error.to_string()))?;

        self.manager
            .as_mut()
            .add_node(node, gbm_device)
            .map_err(|error| GbmGlesRendererError::Renderer(error.to_string()))
    }

    /// Removes the renderer associated with a disconnected DRM node.
    pub fn remove_device(&mut self, node: DrmNode) {
        self.manager.as_mut().remove_node(&node);
    }

    /// Runs an operation with a renderer attached to the requested DRM node.
    ///
    /// The callback must finish before this method returns because Smithay
    /// lends the renderer from its GPU manager.
    pub fn with_renderer<T>(
        &mut self,
        node: DrmNode,
        operation: impl FnOnce(&mut GbmGlesRenderer<'_>) -> T,
    ) -> Result<T, GbmGlesRendererError> {
        let mut renderer = self
            .manager
            .single_renderer(&node)
            .map_err(|error| GbmGlesRendererError::Renderer(error.to_string()))?;

        Ok(operation(&mut renderer))
    }

    /// Returns the number of DRM nodes with initialized GLES renderers.
    pub fn renderer_count(&mut self) -> Result<usize, GbmGlesRendererError> {
        self.manager
            .devices()
            .map(|devices| devices.count())
            .map_err(|error| GbmGlesRendererError::Renderer(error.to_string()))
    }
}
