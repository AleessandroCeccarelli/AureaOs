// SPDX-License-Identifier: GPL-3.0-or-later
// AureaOS — Color Management

//! Reusable, non-destructive color profile configurations.

use crate::{ColorAdjustment, ColorAdjustmentError, IccProfileFingerprint};

/// Stable caller-provided identifier for a saved color configuration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColorConfigurationId(String);

impl ColorConfigurationId {
    /// Creates an identifier from a non-empty value.
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if value.trim().is_empty() || value.len() > 128 {
            return None;
        }
        Some(Self(value))
    }

    /// Returns the identifier value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A named profile selection and its non-destructive user adjustments.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorProfileConfiguration {
    id: ColorConfigurationId,
    name: String,
    profile: IccProfileFingerprint,
    adjustment: ColorAdjustment,
}

impl ColorProfileConfiguration {
    /// Creates a configuration after validating its name and adjustments.
    pub fn new(
        id: ColorConfigurationId,
        name: impl Into<String>,
        profile: IccProfileFingerprint,
        adjustment: ColorAdjustment,
    ) -> Result<Self, ColorConfigurationError> {
        let name = name.into().trim().to_owned();
        if name.is_empty() {
            return Err(ColorConfigurationError::EmptyName);
        }
        if name.chars().count() > 80 {
            return Err(ColorConfigurationError::NameTooLong);
        }
        adjustment
            .validate()
            .map_err(ColorConfigurationError::InvalidAdjustment)?;

        Ok(Self {
            id,
            name,
            profile,
            adjustment,
        })
    }

    pub fn id(&self) -> &ColorConfigurationId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn profile(&self) -> &IccProfileFingerprint {
        &self.profile
    }

    pub fn adjustment(&self) -> ColorAdjustment {
        self.adjustment
    }
}

/// Validation errors for saved color configurations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorConfigurationError {
    EmptyName,
    NameTooLong,
    InvalidAdjustment(ColorAdjustmentError),
    DuplicateId,
    ConfigurationNotFound,
    IdMismatch,
}
