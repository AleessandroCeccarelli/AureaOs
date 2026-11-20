// SPDX-License-Identifier: GPL-3.0-or-later
// AureaOS — Color Management

//! In-memory registry for validated color profile configurations.

use std::collections::BTreeMap;

use crate::{ColorConfigurationError, ColorConfigurationId, ColorProfileConfiguration};

/// Coordinates creation, replacement and removal of named color configurations.
///
/// Persistence and display assignment are intentionally handled by higher-level
/// services. The registry never modifies the referenced ICC profile bytes.
#[derive(Debug, Default)]
pub struct ColorConfigurationManager {
    configurations: BTreeMap<ColorConfigurationId, ColorProfileConfiguration>,
}

impl ColorConfigurationManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new configuration; existing identifiers are never overwritten.
    pub fn create(
        &mut self,
        configuration: ColorProfileConfiguration,
    ) -> Result<(), ColorConfigurationError> {
        if self.configurations.contains_key(configuration.id()) {
            return Err(ColorConfigurationError::DuplicateId);
        }
        self.configurations
            .insert(configuration.id().clone(), configuration);
        Ok(())
    }

    /// Replaces a configuration while preserving its stable identifier.
    pub fn update(
        &mut self,
        id: &ColorConfigurationId,
        configuration: ColorProfileConfiguration,
    ) -> Result<(), ColorConfigurationError> {
        if !self.configurations.contains_key(id) {
            return Err(ColorConfigurationError::ConfigurationNotFound);
        }
        if configuration.id() != id {
            return Err(ColorConfigurationError::IdMismatch);
        }
        self.configurations.insert(id.clone(), configuration);
        Ok(())
    }

    pub fn get(&self, id: &ColorConfigurationId) -> Option<&ColorProfileConfiguration> {
        self.configurations.get(id)
    }

    /// Returns configurations in stable identifier order.
    pub fn list(&self) -> Vec<&ColorProfileConfiguration> {
        self.configurations.values().collect()
    }

    pub fn remove(
        &mut self,
        id: &ColorConfigurationId,
    ) -> Result<ColorProfileConfiguration, ColorConfigurationError> {
        self.configurations
            .remove(id)
            .ok_or(ColorConfigurationError::ConfigurationNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::ColorConfigurationManager;
    use crate::{
        ColorAdjustment, ColorConfigurationError, ColorConfigurationId, ColorProfileConfiguration,
        fingerprint_profile,
    };

    fn profile_fingerprint() -> crate::IccProfileFingerprint {
        let mut bytes = vec![0_u8; 132];
        bytes[0..4].copy_from_slice(&132_u32.to_be_bytes());
        bytes[8..12].copy_from_slice(b"\x04\x30\x00\x00");
        bytes[12..16].copy_from_slice(b"mntr");
        bytes[16..20].copy_from_slice(b"RGB ");
        bytes[20..24].copy_from_slice(b"XYZ ");
        bytes[36..40].copy_from_slice(b"acsp");
        fingerprint_profile(&bytes).unwrap()
    }

    fn configuration(id: &str, name: &str) -> ColorProfileConfiguration {
        ColorProfileConfiguration::new(
            ColorConfigurationId::new(id).unwrap(),
            name,
            profile_fingerprint(),
            ColorAdjustment::NEUTRAL,
        )
        .unwrap()
    }

    #[test]
    fn creates_and_reads_configuration() {
        let mut manager = ColorConfigurationManager::new();
        let config = configuration("office", "Office");
        let id = config.id().clone();

        manager.create(config).unwrap();

        assert_eq!(manager.get(&id).unwrap().name(), "Office");
        assert_eq!(manager.list().len(), 1);
    }

    #[test]
    fn duplicate_identifiers_are_not_overwritten() {
        let mut manager = ColorConfigurationManager::new();
        manager.create(configuration("work", "Work")).unwrap();

        assert_eq!(
            manager.create(configuration("work", "Changed")),
            Err(ColorConfigurationError::DuplicateId)
        );
        assert_eq!(
            manager
                .get(&ColorConfigurationId::new("work").unwrap())
                .unwrap()
                .name(),
            "Work"
        );
    }

    #[test]
    fn update_requires_existing_matching_identifier() {
        let mut manager = ColorConfigurationManager::new();
        let id = ColorConfigurationId::new("work").unwrap();
        manager.create(configuration("work", "Work")).unwrap();

        manager
            .update(&id, configuration("work", "Workroom"))
            .unwrap();
        assert_eq!(manager.get(&id).unwrap().name(), "Workroom");

        assert_eq!(
            manager.update(&id, configuration("other", "Other")),
            Err(ColorConfigurationError::IdMismatch)
        );
    }

    #[test]
    fn removing_configuration_does_not_remove_profile_data() {
        let mut manager = ColorConfigurationManager::new();
        let config = configuration("photo", "Photo");
        let id = config.id().clone();
        let fingerprint = config.profile().clone();
        manager.create(config).unwrap();

        let removed = manager.remove(&id).unwrap();

        assert_eq!(removed.profile(), &fingerprint);
        assert!(manager.get(&id).is_none());
    }

    #[test]
    fn rejects_empty_and_overlong_names() {
        assert_eq!(
            ColorProfileConfiguration::new(
                ColorConfigurationId::new("empty").unwrap(),
                "   ",
                profile_fingerprint(),
                ColorAdjustment::NEUTRAL
            ),
            Err(ColorConfigurationError::EmptyName)
        );
        assert_eq!(
            ColorProfileConfiguration::new(
                ColorConfigurationId::new("long").unwrap(),
                "x".repeat(81),
                profile_fingerprint(),
                ColorAdjustment::NEUTRAL
            ),
            Err(ColorConfigurationError::NameTooLong)
        );
    }
}
