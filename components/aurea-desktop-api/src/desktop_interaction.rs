// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Semantic interactions exchanged between the compositor and desktop shell.

use std::sync::mpsc::{self, Receiver, SendError, Sender, TryRecvError};

/// A desktop position expressed in logical output coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DesktopPosition {
    pub x: i32,
    pub y: i32,
}

/// A desktop-level intent resolved from a physical input gesture.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DesktopInteraction {
    WorkspaceChanged { index: u8 },
    ShowOverview,
    ShowAppExpose,
    ShowContextMenu { position: DesktopPosition },
    /// Dismisses the desktop context menu without affecting other panels.
    DismissContextMenu,
    DesktopItemSelected { id: String },
    DesktopSelectionCleared,
    /// Activates a desktop item after a recognized double-click.
    DesktopItemActivated { id: String },
}

/// Shared, thread-safe selection state sampled by compositor render frames.
///
/// The interaction sender publishes selection changes immediately, and the
/// shell mirrors them into its desktop state. The compositor reads an owned
/// snapshot so rendering never holds the lock during GPU preparation.
#[derive(Debug, Clone, Default)]
pub struct DesktopSelectionController {
    selected_item_id: std::sync::Arc<std::sync::RwLock<Option<String>>>,
}

impl DesktopSelectionController {
    /// Publishes the selected desktop item, or clears selection with None.
    pub fn set_selected_item(&self, item_id: Option<String>) {
        *self
            .selected_item_id
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = item_id;
    }

    /// Returns an owned selection snapshot without exposing the lock.
    pub fn selected_item(&self) -> Option<String> {
        self.selected_item_id
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

/// Sends semantic interactions from the compositor to the shell.
#[derive(Clone)]
pub struct DesktopInteractionSender {
    sender: Sender<DesktopInteraction>,
    selection: DesktopSelectionController,
    context_menu_open: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl DesktopInteractionSender {
    /// Returns the shared desktop selection projection.
    pub fn selection_controller(&self) -> DesktopSelectionController {
        self.selection.clone()
    }

    /// Returns whether the desktop context menu is currently requested.
    pub fn context_menu_is_open(&self) -> bool {
        self.context_menu_open.load(std::sync::atomic::Ordering::Acquire)
    }

    /// Sends one completed desktop interaction.
    pub fn send(
        &self,
        interaction: DesktopInteraction,
    ) -> Result<(), SendError<DesktopInteraction>> {
        let selection = match &interaction {
            DesktopInteraction::DesktopItemSelected { id } => Some(Some(id.clone())),
            DesktopInteraction::DesktopSelectionCleared => Some(None),
            _ => None,
        };
        match &interaction {
            DesktopInteraction::ShowContextMenu { .. } => self.context_menu_open.store(true, std::sync::atomic::Ordering::Release),
            DesktopInteraction::DismissContextMenu => self.context_menu_open.store(false, std::sync::atomic::Ordering::Release),
            _ => {}
        }
        let result = self.sender.send(interaction);
        if result.is_ok() {
            if let Some(selection) = selection {
                self.selection.set_selected_item(selection);
            }
        }
        result
    }
}

/// Receives semantic interactions on the shell thread.
pub struct DesktopInteractionReceiver {
    receiver: Receiver<DesktopInteraction>,
    selection: DesktopSelectionController,
    context_menu_open: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl DesktopInteractionReceiver {
    /// Returns the shared desktop selection projection.
    pub fn selection_controller(&self) -> DesktopSelectionController {
        self.selection.clone()
    }

    /// Receives the next interaction without blocking.
    pub fn try_recv(&self) -> Result<DesktopInteraction, TryRecvError> {
        self.receiver.try_recv()
    }
}

/// Creates the compositor-to-shell interaction channel.
pub fn desktop_interaction_channel() -> (DesktopInteractionSender, DesktopInteractionReceiver) {
    let (sender, receiver) = mpsc::channel();
    let selection = DesktopSelectionController::default();
    let context_menu_open = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    (
        DesktopInteractionSender {
            sender,
            selection: selection.clone(),
        },
        DesktopInteractionReceiver { receiver, selection },
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
    fn shell_and_compositor_share_selection_without_sharing_mutable_guards() {
        let (sender, receiver) = desktop_interaction_channel();
        let shell_selection = receiver.selection_controller();
        let compositor_selection = sender.selection_controller();

        shell_selection.set_selected_item(Some("system:trash".to_owned()));
        assert_eq!(compositor_selection.selected_item().as_deref(), Some("system:trash"));

        compositor_selection.set_selected_item(None);
        assert_eq!(shell_selection.selected_item(), None);
    }

    #[test]
    fn sending_selection_publishes_frame_state_without_waiting_for_shell_poll() {
        let (sender, receiver) = desktop_interaction_channel();

        sender
            .send(DesktopInteraction::DesktopItemSelected {
                id: "system:trash".to_owned(),
            })
            .unwrap();

        assert_eq!(
            sender.selection_controller().selected_item().as_deref(),
            Some("system:trash")
        );
        assert_eq!(
            receiver.try_recv(),
            Ok(DesktopInteraction::DesktopItemSelected {
                id: "system:trash".to_owned(),
            })
        );

        sender
            .send(DesktopInteraction::DesktopSelectionCleared)
            .unwrap();
        assert_eq!(sender.selection_controller().selected_item(), None);
    }

    #[test]
    fn receiver_reports_disconnected_sender() {
        let (sender, receiver) = desktop_interaction_channel();
        drop(sender);

        assert_eq!(receiver.try_recv(), Err(TryRecvError::Disconnected));
    }
}
