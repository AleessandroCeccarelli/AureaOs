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
        // Configure the popup using the geometry prescribed by xdg_positioner.
        // Output-aware constraint adjustment is handled by the popup placement
        // integration; this initial protocol step must still happen so clients
        // can complete their initial configure/commit sequence.
        surface.with_pending_state(|state| {
            state.positioner = positioner;
            state.geometry = positioner.get_geometry();
        });

        if let Err(error) = self.popup_manager().track_popup(PopupKind::Xdg(surface.clone())) {
            tracing::warn!(?error, "Unable to register new XDG popup");
            return;
        }

        if let Err(error) = surface.send_configure() {
            tracing::warn!(?error, "Unable to configure new XDG popup");
            return;
        }

        self.request_all_output_frames();
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {
        // Popup grabs require a seat-owned PopupManager and coordinated pointer
        // and keyboard grabs. They are deliberately not emulated with a partial
        // pointer-only grab; that integration is tracked separately.
    }

    fn reposition_request(
        &mut self,
        surface: PopupSurface,
        positioner: PositionerState,
        token: u32,
    ) {
        // Preserve the protocol-provided positioner and acknowledge the request.
        // The final geometry will gain output-aware constraint adjustment when
        // popup placement is integrated with the output layout.
        surface.with_pending_state(|state| {
            state.positioner = positioner;
            state.geometry = positioner.get_geometry();
        });

        surface.send_repositioned(token);
        self.request_all_output_frames();
    }
}

delegate_xdg_shell!(AureaCompositor);
