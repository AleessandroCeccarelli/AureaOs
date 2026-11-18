// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Capture Core
// Copyright (C) 2026 Alessandro Ceccarelli

//! Validated, backend-independent capture requests.

use crate::CaptureError;

/// Stable identifier for a compositor output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DisplayId(pub u64);

/// Stable identifier for a managed toplevel window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub u64);

/// A rectangle in desktop logical coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureRegion {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl CaptureRegion {
    /// Creates a non-empty region.
    pub fn new(
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Result<Self, CaptureError> {
        if width == 0 || height == 0 {
            return Err(CaptureError::InvalidRegion);
        }

        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }

    pub const fn x(self) -> i32 {
        self.x
    }

    pub const fn y(self) -> i32 {
        self.y
    }

    pub const fn width(self) -> u32 {
        self.width
    }

    pub const fn height(self) -> u32 {
        self.height
    }
}

/// Source selected by the user for a capture session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureTarget {
    Display(DisplayId),
    ActiveWindow,
    Window(WindowId),
    Region(CaptureRegion),
}

/// Audio sources selected for a recording.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioCaptureMode {
    #[default]
    None,
    System,
    Microphone,
    SystemAndMicrophone,
}

/// The operation requested by the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureKind {
    Screenshot,
    Recording { audio: AudioCaptureMode },
}

/// A validated request, before any permission is granted or source is opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureRequest {
    kind: CaptureKind,
    target: CaptureTarget,
    include_cursor: bool,
}

impl CaptureRequest {
    pub const fn new(
        kind: CaptureKind,
        target: CaptureTarget,
        include_cursor: bool,
    ) -> Self {
        Self {
            kind,
            target,
            include_cursor,
        }
    }

    pub const fn kind(self) -> CaptureKind {
        self.kind
    }

    pub const fn target(self) -> CaptureTarget {
        self.target
    }

    pub const fn includes_cursor(self) -> bool {
        self.include_cursor
    }
}

/// Permission decision returned by the trusted desktop authorization layer.
///
/// Construct this value only after the user has approved the corresponding
/// capture request. Screen, system-audio and microphone grants are independent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CapturePermissionGrant {
    screen: bool,
    system_audio: bool,
    microphone: bool,
}

impl CapturePermissionGrant {
    pub const fn new(screen: bool, system_audio: bool, microphone: bool) -> Self {
        Self {
            screen,
            system_audio,
            microphone,
        }
    }

    pub const fn allows_screen(self) -> bool {
        self.screen
    }

    pub const fn allows_system_audio(self) -> bool {
        self.system_audio
    }

    pub const fn allows_microphone(self) -> bool {
        self.microphone
    }
}
