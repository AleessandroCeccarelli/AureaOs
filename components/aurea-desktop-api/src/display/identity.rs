// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Physical display identity evidence and conservative matching policy.

use super::DisplayId;

/// Hardware evidence used to associate a runtime output with a physical display.
///
/// The connector path scopes the observation to a hardware port. EDID fields
/// distinguish models and, when trustworthy, individual units. This value is
/// evidence only: callers must still detect duplicates before restoring state.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayIdentityEvidence {
    connector_path: String,
    manufacturer: [u8; 3],
    product_code: u16,
    serial: Option<String>,
}

impl DisplayIdentityEvidence {
    /// Creates validated identity evidence.
    ///
    /// Empty connector paths and malformed manufacturer codes are rejected.
    /// A serial is retained only when it contains useful non-whitespace data.
    pub fn new(
        connector_path: impl Into<String>,
        manufacturer: [u8; 3],
        product_code: u16,
        serial: Option<String>,
    ) -> Option<Self> {
        let connector_path = connector_path.into();
        if connector_path.trim().is_empty()
            || !manufacturer
                .iter()
                .all(|byte| byte.is_ascii_uppercase())
        {
            return None;
        }

        let serial = serial.and_then(|value| {
            let value = value.trim().to_owned();
            (!value.is_empty() && value != "0").then_some(value)
        });

        Some(Self {
            connector_path,
            manufacturer,
            product_code,
            serial,
        })
    }

    pub fn connector_path(&self) -> &str {
        &self.connector_path
    }

    pub const fn manufacturer(&self) -> [u8; 3] {
        self.manufacturer
    }

    pub const fn product_code(&self) -> u16 {
        self.product_code
    }

    pub fn serial(&self) -> Option<&str> {
        self.serial.as_deref()
    }

    /// Serial evidence permits automatic matching only after duplicate checks.
    pub fn has_unit_serial(&self) -> bool {
        self.serial.is_some()
    }
}

/// Outcome of matching a remembered physical display against current outputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayIdentityMatch {
    /// Exactly one current output has matching unit-level evidence.
    Unique(DisplayId),
    /// More than one output has the same unit-level evidence.
    Ambiguous,
    /// Evidence is insufficient for unattended restoration.
    NeedsReview,
    /// No current output matches the remembered evidence.
    NotFound,
}

/// Resolves a remembered identity conservatively against current observations.
pub struct DisplayIdentityResolver;

impl DisplayIdentityResolver {
    /// Resolves only identities with a non-empty unit serial and exactly one
    /// matching current output. Connector/model-only matches require review.
    pub fn resolve(
        remembered: &DisplayIdentityEvidence,
        current: &[(DisplayId, DisplayIdentityEvidence)],
    ) -> DisplayIdentityMatch {
        if !remembered.has_unit_serial() {
            return DisplayIdentityMatch::NeedsReview;
        }

        let mut matches = current.iter().filter(|(_, evidence)| {
            evidence.manufacturer == remembered.manufacturer
                && evidence.product_code == remembered.product_code
                && evidence.serial == remembered.serial
        });

        let Some((display_id, _)) = matches.next() else {
            return DisplayIdentityMatch::NotFound;
        };

        if matches.next().is_some() {
            return DisplayIdentityMatch::Ambiguous;
        }

        DisplayIdentityMatch::Unique(*display_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{DisplayIdentityEvidence, DisplayIdentityMatch, DisplayIdentityResolver};
    use crate::display::DisplayId;

    fn evidence(connector: &str, serial: Option<&str>) -> DisplayIdentityEvidence {
        DisplayIdentityEvidence::new(
            connector,
            *b"DEL",
            0x1234,
            serial.map(str::to_owned),
        )
        .unwrap()
    }

    #[test]
    fn rejects_invalid_connector_and_manufacturer_evidence() {
        assert!(DisplayIdentityEvidence::new("", *b"DEL", 1, None).is_none());
        assert!(DisplayIdentityEvidence::new("pci/DP-1", *b"DeL", 1, None).is_none());
    }

    #[test]
    fn normalizes_empty_serial_to_missing() {
        assert!(!evidence("pci/DP-1", Some("  ")).has_unit_serial());
        assert!(!evidence("pci/DP-1", Some("0")).has_unit_serial());
    }

    #[test]
    fn resolves_unique_unit_serial_across_connector_changes() {
        let remembered = evidence("pci/DP-1", Some("SN-42"));
        let current = vec![(DisplayId(7), evidence("pci/DP-2", Some("SN-42")))];

        assert_eq!(
            DisplayIdentityResolver::resolve(&remembered, &current),
            DisplayIdentityMatch::Unique(DisplayId(7))
        );
    }

    #[test]
    fn refuses_ambiguous_duplicate_serials() {
        let remembered = evidence("pci/DP-1", Some("SN-42"));
        let current = vec![
            (DisplayId(7), evidence("pci/DP-2", Some("SN-42"))),
            (DisplayId(8), evidence("pci/HDMI-A-1", Some("SN-42"))),
        ];

        assert_eq!(
            DisplayIdentityResolver::resolve(&remembered, &current),
            DisplayIdentityMatch::Ambiguous
        );
    }

    #[test]
    fn connector_and_model_without_serial_require_review() {
        let remembered = evidence("pci/DP-1", None);
        let current = vec![(DisplayId(7), evidence("pci/DP-1", None))];

        assert_eq!(
            DisplayIdentityResolver::resolve(&remembered, &current),
            DisplayIdentityMatch::NeedsReview
        );
    }

    #[test]
    fn reports_missing_unit_identity() {
        let remembered = evidence("pci/DP-1", Some("SN-42"));
        assert_eq!(
            DisplayIdentityResolver::resolve(&remembered, &[]),
            DisplayIdentityMatch::NotFound
        );
    }
}
