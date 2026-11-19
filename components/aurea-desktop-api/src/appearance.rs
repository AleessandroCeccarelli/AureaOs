// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Shared appearance and accessibility preferences for AureaOS applications.
//!
//! Actual color values remain owned by AureaOS Human Interfaces tokens. This
//! contract exposes stable semantic roles, not duplicated palette literals.

use std::sync::mpsc::Receiver;

/// User's requested appearance policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemePreference {
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
    PrimaryText,
    SecondaryText,
    MutedText,
    PrimaryAccent,
    StrongAccent,
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
            Self::PrimaryText => "--aurea-text-primary",
            Self::SecondaryText => "--aurea-text-secondary",
            Self::MutedText => "--aurea-text-muted",
            Self::PrimaryAccent => "--aurea-accent-primary",
            Self::StrongAccent => "--aurea-accent-strong",
            Self::SubtleBorder => "--aurea-border-subtle",
            Self::StrongBorder => "--aurea-border-strong",
            Self::Focus => "--aurea-focus",
            Self::Success => "--aurea-success",
            Self::Warning => "--aurea-warning",
            Self::Critical => "--aurea-critical",
        }
    }
}

/// Appearance state delivered consistently to compatible applications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppearanceSnapshot {
    pub preference: ThemePreference,
    pub color_scheme: ColorScheme,
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
