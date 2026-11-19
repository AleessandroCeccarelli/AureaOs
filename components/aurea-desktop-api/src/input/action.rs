// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Hardware-independent semantic actions produced by input devices.

/// A system action independent from a keyboard or pointing-device code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputAction {
    VolumeUp,
    VolumeDown,
    AudioMuteToggle,
    MediaPlayPause,
    MediaStop,
    MediaPrevious,
    MediaNext,
    DisplayBrightnessUp,
    DisplayBrightnessDown,
    KeyboardBrightnessUp,
    KeyboardBrightnessDown,
    KeyboardBrightnessToggle,
    OpenEmojiPicker,
    StartDictation,
}
