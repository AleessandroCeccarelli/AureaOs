// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Hardware-independent semantic actions produced by input devices.

/// A system action independent from a keyboard manufacturer's key codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyboardAction {
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
