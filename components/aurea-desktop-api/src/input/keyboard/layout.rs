// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Software-managed keyboard layout selection contracts.

use std::sync::mpsc::Receiver;

/// Stable XKB-compatible layout identifier, for example "it" or "us".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyboardLayoutId {
    layout: String,
    variant: Option<String>,
}

impl KeyboardLayoutId {
    /// Creates a layout identifier after validating its basic syntax.
    pub fn new(layout: impl Into<String>, variant: Option<String>) -> Option<Self> {
        let layout = layout.into();
        if layout.is_empty()
            || !layout.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
            || variant.as_ref().is_some_and(|value| {
                value.is_empty()
                    || !value.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
            })
        {
            return None;
        }

        Some(Self { layout, variant })
    }

    pub fn layout(&self) -> &str {
        &self.layout
    }

    pub fn variant(&self) -> Option<&str> {
        self.variant.as_deref()
    }
}

/// User-visible information about an available software layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardLayoutInfo {
    pub id: KeyboardLayoutId,
    pub display_name: String,
    pub language_tag: Option<String>,
}

/// Origin of a non-binding layout suggestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardLayoutHintSource {
    SystemLocale,
    DeviceMetadata,
    PreviousUserChoice,
    VendorExtension,
}

/// A suggested layout. Suggestions must be confirmed before they are applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardLayoutSuggestion {
    pub layout: KeyboardLayoutId,
    pub source: KeyboardLayoutHintSource,
}

/// User-selected input source and optional device override.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardLayoutSelection {
    pub active: KeyboardLayoutId,
    pub device_override: Option<(super::super::InputDeviceId, KeyboardLayoutId)>,
}

/// Confidence assigned to a non-binding software layout detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyboardLayoutDetectionConfidence {
    Low,
    Medium,
    High,
}

/// A layout candidate detected for one physical keyboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardLayoutDetection {
    pub device: super::super::InputDeviceId,
    pub suggestion: KeyboardLayoutSuggestion,
    pub confidence: KeyboardLayoutDetectionConfidence,
}

/// Resolves likely keyboard layouts from trusted metadata and optional extensions.
///
/// Implementations must return suggestions only: applying a detected layout
/// always requires an explicit user selection. Raw keystrokes must not be
/// collected or retained for layout detection.
pub trait KeyboardLayoutDetector: Send + Sync {
    fn detect(
        &self,
        device: &super::super::InputDeviceDescriptor,
        system_locale: Option<&str>,
    ) -> Vec<KeyboardLayoutDetection>;
}

/// Failure while resolving or applying a keyboard layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardLayoutError {
    Unavailable,
    InvalidConfiguration,
    BackendFailure,
}

/// Backend contract for listing, observing and changing software-managed layouts.
pub trait KeyboardLayoutManager: Send + Sync {
    fn available_layouts(&self) -> Vec<KeyboardLayoutInfo>;
    fn suggested_layouts(&self, system_locale: Option<&str>) -> Vec<KeyboardLayoutSuggestion>;
    fn active_selection(&self) -> KeyboardLayoutSelection;
    fn select_layout(&self, layout: KeyboardLayoutId) -> Result<(), KeyboardLayoutError>;
    fn subscribe(&self) -> Receiver<KeyboardLayoutSelection>;
}

#[cfg(test)]
mod tests {
    use super::{KeyboardLayoutHintSource, KeyboardLayoutId, KeyboardLayoutSuggestion};

    #[test]
    fn accepts_common_xkb_layout_identifiers() {
        assert!(KeyboardLayoutId::new("it", None).is_some());
        assert!(KeyboardLayoutId::new("us", Some("intl".to_owned())).is_some());
    }

    #[test]
    fn rejects_empty_or_invalid_layout_identifiers() {
        assert!(KeyboardLayoutId::new("", None).is_none());
        assert!(KeyboardLayoutId::new("it;rm -rf", None).is_none());
        assert!(KeyboardLayoutId::new("us", Some("bad variant".to_owned())).is_none());
    }

    #[test]
    fn detection_confidence_is_ordered_without_applying_a_layout() {
        assert!(KeyboardLayoutDetectionConfidence::High > KeyboardLayoutDetectionConfidence::Medium);
        assert!(KeyboardLayoutDetectionConfidence::Medium > KeyboardLayoutDetectionConfidence::Low);
    }

    #[test]
    fn suggestions_are_explicitly_non_binding_values() {
        let suggestion = KeyboardLayoutSuggestion {
            layout: KeyboardLayoutId::new("it", None).expect("valid layout"),
            source: KeyboardLayoutHintSource::SystemLocale,
        };
        assert_eq!(suggestion.source, KeyboardLayoutHintSource::SystemLocale);
        assert_eq!(suggestion.layout.layout(), "it");
    }
}
