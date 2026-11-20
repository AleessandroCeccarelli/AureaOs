// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Publication and removal of Wayland output globals.

use smithay::reexports::wayland_server::DisplayHandle;

use crate::output::{
    OutputId,
    wayland::{OutputPublicationError, publish_output, publish_outputs},
};

use super::{AureaCompositor, window_placement::center_first_mapped_window};

fn retry_initial_window_placement(compositor: &mut AureaCompositor) {
    if compositor.window_manager.mapped_window_count() != 1 {
        return;
    }

    let Some(surface) = compositor
        .window_manager
        .mapped_toplevels()
        .next()
        .map(|toplevel| toplevel.wl_surface().clone())
    else {
        return;
    };

    center_first_mapped_window(compositor, &surface);
    compositor.refresh_pointer_focus_at_current_location();
}

impl AureaCompositor {
    /// Publishes all registered outputs as Wayland globals.
    ///
    /// All outputs are validated before their globals are created.
    /// Existing Wayland outputs are replaced only after publication succeeds.
    pub fn publish_wayland_outputs(
        &mut self,
        display: &DisplayHandle,
    ) -> Result<(), OutputPublicationError> {
        let had_published_outputs = !self.wayland_outputs.is_empty();
        let outputs = publish_outputs(display, &self.output_manager)?;

        let output_ids = self
            .output_manager
            .registry()
            .iter()
            .filter(|output| output.is_enabled())
            .map(|output| output.id())
            .collect::<Vec<_>>();

        let previous_ids = self.wayland_outputs.keys().copied().collect::<Vec<_>>();
        for output_id in previous_ids {
            self.forget_output_membership(output_id);
        }

        self.wayland_outputs = output_ids.into_iter().zip(outputs).collect();
        if !had_published_outputs {
            retry_initial_window_placement(self);
        }
        self.reconcile_surface_outputs();

        Ok(())
    }

    /// Publishes one registered output as a Wayland global.
    ///
    /// An already published output is left unchanged.
    /// Disabled outputs are not published.
    pub fn publish_wayland_output(
        &mut self,
        display: &DisplayHandle,
        output_id: OutputId,
    ) -> Result<(), OutputPublicationError> {
        if self.wayland_outputs.contains_key(&output_id) {
            return Ok(());
        }

        let Some(output) = self.output_manager.registry().get(output_id) else {
            return Ok(());
        };

        let position = self
            .output_manager
            .layout()
            .get(output_id)
            .unwrap_or_default();

        if let Some(wayland_output) = publish_output(display, output, position)? {
            self.wayland_outputs.insert(output_id, wayland_output);
            retry_initial_window_placement(self);
            self.reconcile_surface_outputs();
        }

        Ok(())
    }

    /// Republishes one output after its description or position changed.
    ///
    /// The replacement is fully validated and created before the retained
    /// output handle is swapped, so a publication error leaves the old global.
    pub fn refresh_wayland_output(
        &mut self,
        display: &DisplayHandle,
        output_id: OutputId,
    ) -> Result<(), OutputPublicationError> {
        let was_published = self.wayland_outputs.contains_key(&output_id);
        let Some(output) = self.output_manager.registry().get(output_id) else {
            self.unpublish_wayland_output(output_id);
            return Ok(());
        };

        if !output.is_enabled() {
            self.unpublish_wayland_output(output_id);
            return Ok(());
        }

        let position = self
            .output_manager
            .layout()
            .get(output_id)
            .unwrap_or_default();

        if let Some(replacement) = publish_output(display, output, position)? {
            // The old Smithay Output owns the previous surface membership set.
            // Emit leave events before replacing its retained handle.
            self.forget_output_membership(output_id);
            self.wayland_outputs.insert(output_id, replacement);
            if !was_published {
                retry_initial_window_placement(self);
            }
        } else {
            self.forget_output_membership(output_id);
            self.wayland_outputs.remove(&output_id);
        }

        self.reconcile_surface_outputs();
        Ok(())
    }

    /// Removes one published Wayland output.
    ///
    /// Dropping the retained Smithay output removes its global.
    /// Returns whether an output was actually removed.
    pub fn unpublish_wayland_output(&mut self, output_id: OutputId) -> bool {
        let removed = self.wayland_outputs.remove(&output_id).is_some();
        if removed {
            self.forget_output_membership(output_id);
            self.reconcile_surface_outputs();
        }
        removed
    }

    /// Returns whether an output currently has a published Wayland global.
    pub fn has_published_wayland_output(&self, output_id: OutputId) -> bool {
        self.wayland_outputs.contains_key(&output_id)
    }
}
