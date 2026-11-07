// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Display output descriptions and registry.

pub mod layout;
pub mod manager;
pub mod wayland;

use std::collections::BTreeMap;

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
        true
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Registry of currently known display outputs.
#[derive(Debug, Default)]
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

#[cfg(test)]
#[path = "../tests/unit/output.rs"]
mod tests;
