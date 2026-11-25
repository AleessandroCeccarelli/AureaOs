// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Public tablet-to-display assignment API.

use aurea_desktop_api::input::{InputDeviceId, InputDeviceKind, InputDeviceProvider};

use crate::{input::pointing::TabletOutputTarget, output::OutputId};

use super::AureaCompositor;

impl AureaCompositor {
    /// Assigns a connected tablet to the full desktop or one enabled output.
    ///
    /// Returns false for unknown/non-tablet devices or disabled outputs.
    pub fn assign_tablet_output(
        &mut self,
        device: InputDeviceId,
        target: TabletOutputTarget,
    ) -> bool {
        if !self
            .pointing_devices()
            .device(device)
            .is_some_and(|descriptor| descriptor.kind == InputDeviceKind::Tablet)
        {
            return false;
        }

        if let TabletOutputTarget::Output(output) = target {
            if !self
                .output_manager
                .registry()
                .get(output)
                .is_some_and(|display| display.is_enabled())
            {
                return false;
            }
        }

        self.tablet_outputs.assign(device, target);
        true
    }

    /// Removes a tablet's explicit mapping, restoring full-desktop mode.
    pub fn unassign_tablet_output(&mut self, device: InputDeviceId) -> bool {
        self.tablet_outputs.unassign(device)
    }

    /// Resolves a tablet against the enabled desktop outputs.
    pub fn tablet_output_target(&self, device: InputDeviceId) -> Option<TabletOutputTarget> {
        let enabled = self
            .output_manager
            .registry()
            .iter()
            .filter(|output| output.is_enabled())
            .map(|output| output.id());

        self.tablet_outputs.resolve(device, enabled)
    }

    /// Returns the enabled output IDs available for tablet mapping.
    pub fn tablet_mapping_outputs(&self) -> Vec<OutputId> {
        self.output_manager
            .registry()
            .iter()
            .filter(|output| output.is_enabled())
            .map(|output| output.id())
            .collect()
    }
}
