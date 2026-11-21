// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Thread-safe requests for live touchpad preference updates.

use std::sync::mpsc::{self, Receiver, SendError, Sender};

use super::{InputDeviceId, TouchpadSettings};

/// A preference update addressed to one connected touchpad.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TouchpadSettingsCommand {
    pub device: InputDeviceId,
    pub settings: TouchpadSettings,
}

/// Shell-facing handle for requesting live touchpad changes.
#[derive(Clone)]
pub struct TouchpadSettingsController {
    sender: Sender<TouchpadSettingsCommand>,
}

impl TouchpadSettingsController {
    /// Queues a profile update for the compositor event loop.
    pub fn set(
        &self,
        device: InputDeviceId,
        settings: TouchpadSettings,
    ) -> Result<(), SendError<TouchpadSettingsCommand>> {
        self.sender.send(TouchpadSettingsCommand { device, settings })
    }
}

/// Creates the shell controller and its compositor-owned command receiver.
pub fn touchpad_settings_channel() -> (
    TouchpadSettingsController,
    Receiver<TouchpadSettingsCommand>,
) {
    let (sender, receiver) = mpsc::channel();
    (TouchpadSettingsController { sender }, receiver)
}

#[cfg(test)]
mod tests {
    use super::touchpad_settings_channel;
    use crate::input::{InputDeviceId, TouchpadSettings};

    #[test]
    fn controller_delivers_device_scoped_settings() {
        let (controller, receiver) = touchpad_settings_channel();
        let settings = TouchpadSettings::new(false, true, true, false, false);

        controller.set(InputDeviceId(7), settings).unwrap();

        let command = receiver.try_recv().unwrap();
        assert_eq!(command.device, InputDeviceId(7));
        assert_eq!(command.settings, settings);
    }
}
