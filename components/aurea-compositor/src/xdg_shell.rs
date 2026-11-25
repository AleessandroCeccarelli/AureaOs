// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli
//! XDG shell protocol handling.
use crate::AureaCompositor;
use smithay::{
    delegate_xdg_shell,
    desktop::PopupKind,
    reexports::wayland_server::protocol::wl_seat,
    utils::Serial,
    wayland::shell::xdg::{
        PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
    },
};

impl XdgShellHandler for AureaCompositor {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        self.xdg_shell_state()
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        self.window_manager().add_toplevel(surface);
    }

    fn new_popup(&mut self, surface: PopupSurface, positioner: PositionerState) {
        // Resolve the output containing the popup anchor, then apply the
        // XDG flip/slide/resize constraint-adjustment policy in parent space.
        let geometry =
            crate::compositor::popup_placement::constrain_geometry(self, &surface, positioner);
        surface.with_pending_state(|state| {
            state.positioner = positioner;
            state.geometry = geometry;
        });

        if let Err(error) = self.popup_manager().track_popup(PopupKind::Xdg(surface.clone())) {
            tracing::warn!(?error, "Unable to register new XDG popup");
            return;
        }
        self.popup_placement_registry
            .register(surface.clone(), positioner);

        if let Err(error) = surface.send_configure() {
            tracing::warn!(?error, "Unable to configure new XDG popup");
            return;
        }

        self.request_all_output_frames();
    }

    fn grab(&mut self, surface: PopupSurface, seat: wl_seat::WlSeat, serial: Serial) {
        if let Err(error) = self.start_popup_grab(PopupKind::Xdg(surface), &seat, serial) {
            tracing::warn!(?error, "Unable to start coordinated XDG popup grab");
        }
    }

    fn reposition_request(
        &mut self,
        surface: PopupSurface,
        positioner: PositionerState,
        token: u32,
    ) {
        // Re-run output selection and XDG constraints for the replacement
        // positioner before acknowledging the client's reposition token.
        let geometry =
            crate::compositor::popup_placement::constrain_geometry(self, &surface, positioner);
        surface.with_pending_state(|state| {
            state.positioner = positioner;
            state.geometry = geometry;
        });
        self.popup_placement_registry
            .update(surface.clone(), positioner);

        surface.send_repositioned(token);
        self.request_all_output_frames();
    }
    
    fn popup_destroyed(&mut self, surface: PopupSurface) {
        self.popup_placement_registry.remove(&surface);
    }
}

delegate_xdg_shell!(AureaCompositor);
