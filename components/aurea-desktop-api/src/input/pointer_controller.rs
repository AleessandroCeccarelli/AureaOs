// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Thread-safe requests for live pointer preference updates.

use std::sync::mpsc::{self, Receiver, SendError, Sender};

use super::{InputDeviceId, PointerSettings};

/// A preference update addressed to one connected physical pointer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerSettingsCommand {
    pub device: InputDeviceId,
    pub settings: PointerSettings,
}

/// Shell-facing handle for requesting live pointer changes.
#[derive(Clone)]
pub struct PointerSettingsController {
    sender: Sender<PointerSettingsCommand>,
}

impl PointerSettingsController {
    /// Queues a profile update for the compositor event loop.
    pub fn set(
        &self,
        device: InputDeviceId,
        settings: PointerSettings,
    ) -> Result<(), SendError<PointerSettingsCommand>> {
        self.sender.send(PointerSettingsCommand { device, settings })
    }
}

/// Creates the shell controller and its compositor-owned command receiver.
pub fn pointer_settings_channel() -> (
    PointerSettingsController,
    Receiver<PointerSettingsCommand>,
) {
    let (sender, receiver) = mpsc::channel();
    (PointerSettingsController { sender }, receiver)
}

#[cfg(test)]
mod tests {
    use super::pointer_settings_channel;
    use crate::input::{InputDeviceId, PointerSettings};

    #[test]
    fn controller_delivers_device_scoped_settings() {
        let (controller, receiver) = pointer_settings_channel();
        let settings = PointerSettings::default();

        controller.set(InputDeviceId(11), settings).unwrap();

        let command = receiver.try_recv().unwrap();
        assert_eq!(command.device, InputDeviceId(11));
        assert_eq!(command.settings, settings);
    }
}
