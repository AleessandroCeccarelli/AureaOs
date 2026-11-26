// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Thread-safe requests for global desktop pointer behavior preferences.

use std::sync::mpsc::{self, Receiver, SendError, Sender};

/// Global pointer behavior preferences, independent from physical devices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopPointerBehaviorSettings {
    double_click_interval_ms: u64,
}

impl Default for DesktopPointerBehaviorSettings {
    fn default() -> Self {
        Self { double_click_interval_ms: 400 }
    }
}

impl DesktopPointerBehaviorSettings {
    pub const MIN_DOUBLE_CLICK_INTERVAL_MS: u64 = 150;
    pub const MAX_DOUBLE_CLICK_INTERVAL_MS: u64 = 1_000;

    /// Creates settings, clamping the interval to the supported range.
    pub fn new(double_click_interval_ms: u64) -> Self {
        Self {
            double_click_interval_ms: double_click_interval_ms.clamp(
                Self::MIN_DOUBLE_CLICK_INTERVAL_MS,
                Self::MAX_DOUBLE_CLICK_INTERVAL_MS,
            ),
        }
    }

    pub fn double_click_interval_ms(self) -> u64 {
        self.double_click_interval_ms
    }
}

/// A global desktop pointer behavior update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopPointerBehaviorCommand {
    pub settings: DesktopPointerBehaviorSettings,
}

/// Shell-facing handle for global desktop pointer behavior preferences.
#[derive(Clone)]
pub struct DesktopPointerBehaviorController {
    sender: Sender<DesktopPointerBehaviorCommand>,
}

impl DesktopPointerBehaviorController {
    pub fn set(
        &self,
        settings: DesktopPointerBehaviorSettings,
    ) -> Result<(), SendError<DesktopPointerBehaviorCommand>> {
        self.sender.send(DesktopPointerBehaviorCommand { settings })
    }
}

/// Creates the shell controller and compositor command receiver.
pub fn desktop_pointer_behavior_channel() -> (
    DesktopPointerBehaviorController,
    Receiver<DesktopPointerBehaviorCommand>,
) {
    let (sender, receiver) = mpsc::channel();
    (DesktopPointerBehaviorController { sender }, receiver)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_400_milliseconds() {
        assert_eq!(DesktopPointerBehaviorSettings::default().double_click_interval_ms(), 400);
    }

    #[test]
    fn clamps_double_click_interval() {
        assert_eq!(DesktopPointerBehaviorSettings::new(1).double_click_interval_ms(), 150);
        assert_eq!(DesktopPointerBehaviorSettings::new(5_000).double_click_interval_ms(), 1_000);
    }

    #[test]
    fn controller_delivers_global_settings() {
        let (controller, receiver) = desktop_pointer_behavior_channel();
        let settings = DesktopPointerBehaviorSettings::new(650);
        controller.set(settings).unwrap();
        assert_eq!(receiver.try_recv().unwrap().settings, settings);
    }
}
