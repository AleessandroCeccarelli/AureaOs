
 // SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli
//! Converts DRM output descriptions into the compositor output model.

use std::{error::Error, fmt};

use crate::output::{
    layout::OutputPosition,
    manager::OutputManager,
    DisplayOutput,
    OutputId,
    OutputMode,
};

/// Describes a display mode reported by the DRM backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrmModeInfo {
    width: u32,
    height: u32,
    refresh_millihz: u32,
}

impl DrmModeInfo {
    /// Creates a display mode description.
    pub const fn new(
        width: u32,
        height: u32,
        refresh_millihz: u32,
    ) -> Self {
        Self {
            width,
            height,
            refresh_millihz,
        }
    }

    /// Returns the mode width in pixels.
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the mode height in pixels.
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns the refresh rate in millihertz.
    pub const fn refresh_millihz(&self) -> u32 {
        self.refresh_millihz
    }

    /// Converts the DRM mode into the compositor's output mode.
    pub fn to_output_mode(self) -> Option<OutputMode> {
        OutputMode::new(
            self.width,
            self.height,
            self.refresh_millihz,
        )
    }
}

/// Describes a physical display output discovered by DRM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrmOutputDescriptor {
    id: OutputId,
    name: String,
    modes: Vec<DrmModeInfo>,
    current_mode: DrmModeInfo,
    scale: u32,
}

impl DrmOutputDescriptor {
    /// Creates a descriptor from DRM output information.
    pub fn new(
        id: OutputId,
        name: impl Into<String>,
        modes: Vec<DrmModeInfo>,
        current_mode: DrmModeInfo,
        scale: u32,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            modes,
            current_mode,
            scale,
        }
    }

    /// Returns the output identifier.
    pub const fn id(&self) -> OutputId {
        self.id
    }

    /// Returns the output name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the supported display modes.
    pub fn modes(&self) -> &[DrmModeInfo] {
        &self.modes
    }

    /// Returns the current display mode.
    pub const fn current_mode(&self) -> DrmModeInfo {
        self.current_mode
    }

    /// Returns the output scale.
    pub const fn scale(&self) -> u32 {
        self.scale
    }

    /// Converts this descriptor into the compositor output model.
    pub fn into_display_output(
        self,
    ) -> Result<DisplayOutput, DrmOutputError> {
        let modes = self
            .modes
            .into_iter()
            .map(DrmModeInfo::to_output_mode)
            .collect::<Option<Vec<_>>>()
            .ok_or(DrmOutputError::InvalidMode)?;

        for index in 0..modes.len() {
            if modes[..index].contains(&modes[index]) {
                return Err(DrmOutputError::DuplicateMode);
            }
        }

        let current_mode = self
            .current_mode
            .to_output_mode()
            .ok_or(DrmOutputError::InvalidMode)?;

        if !modes.contains(&current_mode) {
            return Err(DrmOutputError::UnsupportedCurrentMode);
        }

        DisplayOutput::new(
            self.id,
            self.name,
            modes,
            current_mode,
            self.scale,
        )
        .ok_or(DrmOutputError::InvalidOutput)
    }
}

/// Errors produced while converting or registering a DRM output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrmOutputError {
    InvalidMode,
    DuplicateMode,
    UnsupportedCurrentMode,
    InvalidOutput,
    DuplicateOutputId,
}

impl fmt::Display for DrmOutputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMode => {
                formatter.write_str("DRM reported an invalid display mode")
            }
            Self::DuplicateMode => {
                formatter.write_str("DRM reported duplicate display modes")
            }
            Self::UnsupportedCurrentMode => {
                formatter.write_str(
                    "The current display mode is not in the supported mode list",
                )
            }
            Self::InvalidOutput => {
                formatter.write_str("The DRM output description is invalid")
            }
            Self::DuplicateOutputId => {
                formatter.write_str("The output ID is already registered")
            }
        }
    }
}

impl Error for DrmOutputError {}

/// Converts and registers a DRM output with its logical position.
pub fn register_output(
    manager: &mut OutputManager,
    descriptor: DrmOutputDescriptor,
    position: OutputPosition,
) -> Result<(), DrmOutputError> {
    let output = descriptor.into_display_output()?;

    manager
        .register(output, (position.x(), position.y()))
        .map_err(|_| DrmOutputError::DuplicateOutputId)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_descriptor() -> DrmOutputDescriptor {
        let mode = DrmModeInfo::new(1920, 1080, 60_000);

        DrmOutputDescriptor::new(
            1,
            "HDMI-A-1",
            vec![mode],
            mode,
            1,
        )
    }

    #[test]
    fn converts_valid_descriptor() {
        let output = valid_descriptor()
            .into_display_output()
            .expect("valid descriptor should convert");

        assert_eq!(output.id(), 1);
        assert_eq!(output.name(), "HDMI-A-1");
        assert_eq!(output.current_mode().width(), 1920);
        assert_eq!(output.current_mode().height(), 1080);
    }

    #[test]
    fn rejects_invalid_mode() {
        let invalid = DrmModeInfo::new(0, 1080, 60_000);

        let descriptor = DrmOutputDescriptor::new(
            1,
            "HDMI-A-1",
            vec![invalid],
            invalid,
            1,
        );

        assert_eq!(
            descriptor.into_display_output(),
            Err(DrmOutputError::InvalidMode),
        );
    }

    #[test]
    fn rejects_duplicate_modes() {
        let mode = DrmModeInfo::new(1920, 1080, 60_000);

        let descriptor = DrmOutputDescriptor::new(
            1,
            "HDMI-A-1",
            vec![mode, mode],
            mode,
            1,
        );

        assert_eq!(
            descriptor.into_display_output(),
            Err(DrmOutputError::DuplicateMode),
        );
    }

    #[test]
    fn rejects_unsupported_current_mode() {
        let supported = DrmModeInfo::new(1920, 1080, 60_000);
        let unsupported = DrmModeInfo::new(1280, 720, 60_000);

        let descriptor = DrmOutputDescriptor::new(
            1,
            "HDMI-A-1",
            vec![supported],
            unsupported,
            1,
        );

        assert_eq!(
            descriptor.into_display_output(),
            Err(DrmOutputError::UnsupportedCurrentMode),
        );
    }

    #[test]
    fn rejects_zero_scale() {
        let mode = DrmModeInfo::new(1920, 1080, 60_000);

        let descriptor = DrmOutputDescriptor::new(
            1,
            "HDMI-A-1",
            vec![mode],
            mode,
            0,
        );

        assert_eq!(
            descriptor.into_display_output(),
            Err(DrmOutputError::InvalidOutput),
        );
    }
}
