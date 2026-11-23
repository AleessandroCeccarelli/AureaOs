// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli
//! XDG shell protocol handling.
use crate::AureaCompositor;
use smithay::{
    delegate_xdg_shell,
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
    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {
        // Popup placement will be implemented with window management.
    }
    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {
        // Interactive popup grabs will be implemented with input handling.
    }
    fn reposition_request(
        &mut self,
        _surface: PopupSurface,
        _positioner: PositionerState,
        _token: u32,
    ) {
        // Popup repositioning will be implemented with window management.
    }
}
delegate_xdg_shell!(AureaCompositor);
