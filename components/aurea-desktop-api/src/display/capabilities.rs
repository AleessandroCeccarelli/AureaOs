// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Display identity and hardware/driver capability descriptions.

use super::modes::DisplayMode;

/// Runtime identifier for a currently discovered display output.
///
/// This identifier is scoped to the active backend topology. It is not a
/// persistent physical-monitor identity and must not key long-lived settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayId(pub u128);

/// Physical or virtual connector type reported by the display backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayConnector {
    Unknown,
    DisplayPort,
    EmbeddedDisplayPort,
    Hdmi,
    UsbC,
    Dvi,
    Vga,
    Virtual,
}

/// Validated effective variable-refresh range, expressed in millihertz.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VariableRefreshRange {
    minimum_millihz: u32,
    maximum_millihz: u32,
}

impl VariableRefreshRange {
    pub const fn new(minimum_millihz: u32, maximum_millihz: u32) -> Option<Self> {
        if minimum_millihz == 0 || maximum_millihz < minimum_millihz {
            return None;
        }

        Some(Self {
            minimum_millihz,
            maximum_millihz,
        })
    }

    pub const fn minimum_millihz(self) -> u32 {
        self.minimum_millihz
    }

    pub const fn maximum_millihz(self) -> u32 {
        self.maximum_millihz
    }
}

/// Physical dimensions reported by the active display capability source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayPhysicalSize {
    pub width_mm: u32,
    pub height_mm: u32,
}

/// Capabilities verified by the active driver and display pipeline.
///
/// A missing value means unknown or unavailable, not an inferred capability.
/// In particular, VRR must not be enabled from EDID marketing data alone.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DisplayCapabilities {
    pub variable_refresh: Option<VariableRefreshRange>,
    /// Unknown until the active display pipeline verifies HDR support.
    pub hdr_supported: Option<bool>,
    pub maximum_bits_per_color: Option<u8>,
    pub physical_size: Option<DisplayPhysicalSize>,
}

/// Public description of a connected or discovered display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayDescriptor {
    pub id: DisplayId,
    pub name: String,
    pub connector: DisplayConnector,
    pub connected: bool,
    pub supported_modes: Vec<DisplayMode>,
    pub current_mode: Option<DisplayMode>,
    pub capabilities: DisplayCapabilities,
}

impl DisplayDescriptor {
    /// Returns true only when the effective capability profile reports VRR.
    pub fn supports_variable_refresh(&self) -> bool {
        self.capabilities.variable_refresh.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::VariableRefreshRange;

    #[test]
    fn rejects_invalid_variable_refresh_ranges() {
        assert!(VariableRefreshRange::new(0, 144_000).is_none());
        assert!(VariableRefreshRange::new(144_000, 60_000).is_none());
        assert_eq!(
            VariableRefreshRange::new(48_000, 144_000)
                .unwrap()
                .maximum_millihz(),
            144_000
        );
    }
}
