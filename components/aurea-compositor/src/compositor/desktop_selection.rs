// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Desktop-canvas item selection from global pointer coordinates.

use aurea_desktop_api::{
    desktop::{DesktopHitTest, DesktopPoint},
    desktop_interaction::DesktopInteraction,
};
use smithay::utils::{Logical, Point};

use super::AureaCompositor;

/// Resolves a primary click on the desktop canvas and informs the shell.
///
/// Window surfaces are handled first by the caller. The shell owns selection
/// presentation and later activation policy; the compositor only performs
/// logical-coordinate hit testing and emits a semantic interaction.
impl AureaCompositor {
    pub(super) fn select_desktop_item_at(&mut self, location: Point<f64, Logical>) {
        let logical = location.to_i32_round::<i32>();
        let selected = DesktopHitTest::item_id_at(
            self.desktop_items.items(),
            DesktopPoint::new(logical.x, logical.y),
            crate::desktop::DESKTOP_ICON_LOGICAL_SIZE,
            crate::desktop::DESKTOP_ICON_LOGICAL_SIZE,
        );

        let interaction = match selected {
            Some(id) => DesktopInteraction::DesktopItemSelected {
                id: id.as_str().to_owned(),
            },
            None => DesktopInteraction::DesktopSelectionCleared,
        };

        if self.desktop_interactions.send(interaction).is_err() {
            tracing::debug!("Shell desktop-selection receiver is no longer available");
        }

        self.request_all_output_frames();
    }
}
