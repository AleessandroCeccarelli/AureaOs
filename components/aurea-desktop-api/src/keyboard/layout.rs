// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Software-managed keyboard layout selection contracts.
//!
//! These types describe user intent. XKB compilation and application belong
//! to the input implementation, not to this technology-independent API.

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

    /// Returns the XKB layout name.
    pub fn layout(&self) -> &str {
        &self.layout
    }

    /// Returns the optional XKB variant.
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

/// User-selected input source and optional device override.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardLayoutSelection {
    pub active: KeyboardLayoutId,
    pub device_override: Option<(u64, KeyboardLayoutId)>,
}

/// Backend contract for listing and changing software-managed layouts.
///
/// Implementations are responsible for applying a valid XKB keymap and
/// preserving coherent key/modifier state during a layout transition.
pub trait KeyboardLayoutManager: Send + Sync {
    /// Lists layouts currently available to the system.
    fn available_layouts(&self) -> Vec<KeyboardLayoutInfo>;

    /// Returns the active session layout.
    fn active_layout(&self) -> KeyboardLayoutId;

    /// Selects a layout for the session.
    fn select_layout(&self, layout: KeyboardLayoutId) -> Result<(), KeyboardLayoutError>;
}

/// Failure while resolving or applying a keyboard layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardLayoutError {
    Unavailable,
    InvalidConfiguration,
    BackendFailure,
}
