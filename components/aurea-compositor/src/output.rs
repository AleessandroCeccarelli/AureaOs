// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Display output descriptions and registry.

pub mod coordinates;
pub mod layout;
pub mod limits;
pub mod manager;
pub mod scale;
pub mod wayland;

use std::collections::BTreeMap;

use smithay::utils::Transform;

use self::scale::ScaleFactor;

/// Stable identifier for a display output.
///
/// The 128-bit value can encode a 64-bit DRM device identifier
/// and a 32-bit connector identifier without collisions.
pub type OutputId = u128;

/// Display mode expressed in pixels and millihertz.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputMode {
    width: u32,
    height: u32,
    refresh_millihz: u32,
}

impl OutputMode {
    /// Creates a mode if its dimensions and refresh rate are valid.
    pub fn new(width: u32, height: u32, refresh_millihz: u32) -> Option<Self> {
        if width == 0 || height == 0 || refresh_millihz == 0 {
            return None;
        }

        Some(Self {
            width,
            height,
            refresh_millihz,
        })
    }

    pub fn width(self) -> u32 {
        self.width
    }

    pub fn height(self) -> u32 {
        self.height
    }

    pub fn refresh_millihz(self) -> u32 {
        self.refresh_millihz
    }
}

/// Description of one connected display output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayOutput {
    id: OutputId,
    name: String,
    modes: Vec<OutputMode>,
    current_mode: OutputMode,
    scale: u32,
    fractional_scale: Option<ScaleFactor>,
    transform: Transform,
    enabled: bool,
}

impl DisplayOutput {
    /// Creates an output with a valid current mode and scale.
    pub fn new(
        id: OutputId,
        name: impl Into<String>,
        modes: Vec<OutputMode>,
        current_mode: OutputMode,
        scale: u32,
    ) -> Option<Self> {
        let name = name.into();

        if name.is_empty() || scale == 0 || modes.is_empty() || !modes.contains(&current_mode) {
            return None;
        }

        Some(Self {
            id,
            name,
            modes,
            current_mode,
            scale,
            fractional_scale: None,
            transform: Transform::Normal,
            enabled: true,
        })
    }

    pub fn id(&self) -> OutputId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn modes(&self) -> &[OutputMode] {
        &self.modes
    }

    pub fn current_mode(&self) -> OutputMode {
        self.current_mode
    }

    pub fn scale(&self) -> u32 {
        self.scale
    }

    /// Returns the configured fractional scale, when one is active.
    pub fn fractional_scale(&self) -> Option<ScaleFactor> {
        self.fractional_scale
    }

    /// Returns the physical orientation of this output.
    pub fn transform(&self) -> Transform {
        self.transform
    }

    /// Sets the physical orientation reported and used by this output.
    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    /// Sets a validated fractional scale and disables the legacy integer scale.
    pub fn set_fractional_scale(&mut self, scale: ScaleFactor) {
        self.fractional_scale = Some(scale);
    }

    /// Returns the effective scale as a floating-point value for geometry APIs.
    pub fn effective_scale(&self) -> f64 {
        self.fractional_scale
            .map_or(f64::from(self.scale), ScaleFactor::as_f64)
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Changes the current mode if it is supported by this output.
    pub fn set_mode(&mut self, mode: OutputMode) -> bool {
        if !self.modes.contains(&mode) {
            return false;
        }

        self.current_mode = mode;
        true
    }

    /// Sets the integer scale factor. Zero is rejected.
    pub fn set_scale(&mut self, scale: u32) -> bool {
        if scale == 0 {
            return false;
        }

        self.scale = scale;
        self.fractional_scale = None;
        true
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Applies newly discovered properties while preserving user settings.
    ///
    /// The preferred mode is retained if it remains supported. Scale and
    /// enabled state are compositor configuration, not DRM discovery data.
    pub(crate) fn update_from_discovery(&mut self, mut discovered: Self) {
        if discovered.modes.contains(&self.current_mode) {
            discovered.current_mode = self.current_mode;
        }

        discovered.scale = self.scale;
        discovered.fractional_scale = self.fractional_scale;
        discovered.transform = self.transform;
        discovered.enabled = self.enabled;

        *self = discovered;
    }
}

/// Registry of currently known display outputs.
#[derive(Debug, Clone, Default)]
pub struct OutputRegistry {
    outputs: BTreeMap<OutputId, DisplayOutput>,
}

impl OutputRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an output. Returns false if its ID is already present.
    pub fn register(&mut self, output: DisplayOutput) -> bool {
        if self.outputs.contains_key(&output.id()) {
            return false;
        }

        self.outputs.insert(output.id(), output);
        true
    }

    /// Removes an output and returns its previous description.
    pub fn remove(&mut self, id: OutputId) -> Option<DisplayOutput> {
        self.outputs.remove(&id)
    }

    pub fn get(&self, id: OutputId) -> Option<&DisplayOutput> {
        self.outputs.get(&id)
    }

    pub fn get_mut(&mut self, id: OutputId) -> Option<&mut DisplayOutput> {
        self.outputs.get_mut(&id)
    }

    pub fn len(&self) -> usize {
        self.outputs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.outputs.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &DisplayOutput> {
        self.outputs.values()
    }
}
