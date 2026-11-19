// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Shared appearance and accessibility preferences for AureaOS applications.
//!
//! Actual color values remain owned by AureaOS Human Interfaces tokens. This
//! contract exposes stable semantic roles, not duplicated palette literals.

use std::sync::mpsc::Receiver;

/// System-managed environmental adaptation, separate from the Light/Dark theme.
pub mod lumen;

/// User's requested appearance policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemePreference {
    /// Follow the system's resolved appearance policy.
    System,
    Light,
    Dark,
}

/// Resolved color scheme currently active for applications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorScheme {
    Light,
    Dark,
}

/// Stable semantic color roles mapped to the canonical Human Interfaces tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticColorRole {
    DesktopSurface,
    WindowSurface,
    PanelSurface,
    ElevatedSurface,
    StrongSurface,
    PrimaryText,
    SecondaryText,
    MutedText,
    PrimaryAccent,
    StrongAccent,
    SoftAccent,
    WarmAccent,
    SoftWarmAccent,
    SubtleBorder,
    StrongBorder,
    Focus,
    Success,
    Warning,
    Critical,
}

impl SemanticColorRole {
    /// Returns the canonical token name from the AureaOS Human Interfaces token file.
    pub const fn token_name(self) -> &'static str {
        match self {
            Self::DesktopSurface => "--aurea-surface-desktop",
            Self::WindowSurface => "--aurea-surface-window",
            Self::PanelSurface => "--aurea-surface-panel",
            Self::ElevatedSurface => "--aurea-surface-elevated",
            Self::StrongSurface => "--aurea-surface-strong",
            Self::PrimaryText => "--aurea-text-primary",
            Self::SecondaryText => "--aurea-text-secondary",
            Self::MutedText => "--aurea-text-muted",
            Self::PrimaryAccent => "--aurea-accent-primary",
            Self::StrongAccent => "--aurea-accent-strong",
            Self::SoftAccent => "--aurea-accent-soft",
            Self::WarmAccent => "--aurea-accent-warm",
            Self::SoftWarmAccent => "--aurea-accent-warm-soft",
            Self::SubtleBorder => "--aurea-border-subtle",
            Self::StrongBorder => "--aurea-border-strong",
            Self::Focus => "--aurea-focus",
            Self::Success => "--aurea-success",
            Self::Warning => "--aurea-warning",
            Self::Critical => "--aurea-critical",
        }
    }
}


/// Identifier of a centrally managed AureaOS semantic palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PaletteId {
    /// The approved blue-gray AureaOS palette with blue and restrained warm accents.
    #[default]
    Aurea,
}

/// Semantic geometry roles mapped to the canonical Human Interfaces form tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormRole {
    Window,
    Panel,
    Dock,
    Control,
    Notification,
}

impl FormRole {
    pub const fn token_name(self) -> &'static str {
        match self {
            Self::Window => "--aurea-window-radius",
            Self::Panel => "--aurea-panel-radius",
            Self::Dock => "--aurea-dock-radius",
            Self::Control => "--aurea-control-radius",
            Self::Notification => "--aurea-notification-radius",
        }
    }
}

/// System-wide visual identity profile shared by compatible applications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FormProfile {
    /// Uses the approved AureaOS geometry tokens; applications cannot substitute
    /// private system-wide shapes.
    pub canonical: bool,
}

/// Appearance state delivered consistently to compatible applications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppearanceSnapshot {
    pub preference: ThemePreference,
    pub color_scheme: ColorScheme,
    pub palette: PaletteId,
    pub form: FormProfile,
    pub reduced_motion: bool,
    pub reduced_transparency: bool,
    pub high_contrast: bool,
}

/// Read-only application contract for system appearance changes.
///
/// Implementations must provide an initial snapshot and notify subscribers
/// whenever the resolved appearance or an accessibility preference changes.
pub trait AppearanceProvider: Send + Sync {
    fn snapshot(&self) -> AppearanceSnapshot;
    fn subscribe(&self) -> Receiver<AppearanceSnapshot>;
}
