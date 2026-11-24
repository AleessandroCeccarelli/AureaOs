// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Semantic interactions exchanged between the compositor and desktop shell.

use std::sync::mpsc::{self, Receiver, SendError, Sender, TryRecvError};

/// A desktop-level intent resolved from a physical input gesture.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DesktopInteraction {
    WorkspaceChanged { index: u8 },
    ShowOverview,
    ShowContextMenu,
    DesktopItemSelected { id: String },
    DesktopSelectionCleared,
}

/// Sends semantic interactions from the compositor to the shell.
#[derive(Clone)]
pub struct DesktopInteractionSender {
    sender: Sender<DesktopInteraction>,
}

impl DesktopInteractionSender {
    /// Sends one completed desktop interaction.
    pub fn send(
        &self,
        interaction: DesktopInteraction,
    ) -> Result<(), SendError<DesktopInteraction>> {
        self.sender.send(interaction)
    }
}

/// Receives semantic interactions on the shell thread.
pub struct DesktopInteractionReceiver {
    receiver: Receiver<DesktopInteraction>,
}

impl DesktopInteractionReceiver {
    /// Receives the next interaction without blocking.
    pub fn try_recv(&self) -> Result<DesktopInteraction, TryRecvError> {
        self.receiver.try_recv()
    }
}

/// Creates the compositor-to-shell interaction channel.
pub fn desktop_interaction_channel() -> (DesktopInteractionSender, DesktopInteractionReceiver) {
    let (sender, receiver) = mpsc::channel();
    (
        DesktopInteractionSender { sender },
        DesktopInteractionReceiver { receiver },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_delivers_interactions_in_order() {
        let (sender, receiver) = desktop_interaction_channel();
        sender
            .send(DesktopInteraction::WorkspaceChanged { index: 1 })
            .unwrap();
        sender.send(DesktopInteraction::ShowOverview).unwrap();

        assert_eq!(
            receiver.try_recv(),
            Ok(DesktopInteraction::WorkspaceChanged { index: 1 })
        );
        assert_eq!(receiver.try_recv(), Ok(DesktopInteraction::ShowOverview));
        assert_eq!(receiver.try_recv(), Err(TryRecvError::Empty));
    }

    #[test]
    fn receiver_reports_disconnected_sender() {
        let (sender, receiver) = desktop_interaction_channel();
        drop(sender);

        assert_eq!(receiver.try_recv(), Err(TryRecvError::Disconnected));
    }
}
