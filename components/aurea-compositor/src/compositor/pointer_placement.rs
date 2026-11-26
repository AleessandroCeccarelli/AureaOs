// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Event-driven pointer placement after desktop output lifecycle changes.

use smithay::{
    input::pointer::MotionEvent,
    utils::{Clock, Monotonic, Point, SERIAL_COUNTER},
};

use super::AureaCompositor;

impl AureaCompositor {
    /// Places the pointer at the center of the first enabled output.
    ///
    /// This is intentionally called only after output startup, session
    /// restoration, or output reconciliation. It does not run in the frame
    /// loop and does not move a pointer while a client owns an active grab.
    pub fn center_pointer_on_primary_output(&mut self) -> bool {
        let Some(pointer) = self.seat.get_pointer() else {
            return false;
        };

        if pointer.is_grabbed() {
            self.pointer_recenter_pending = true;
            return false;
        }

        let target = self
            .output_manager
            .registry()
            .iter()
            .filter(|output| output.is_enabled())
            .find_map(|output| {
                let bounds = self.output_manager.logical_bounds(output.id()).ok()??;
                let origin = bounds.origin();
                let size = bounds.size();

                Some(Point::from((
                    origin.x as f64 + f64::from(size.width) / 2.0,
                    origin.y as f64 + f64::from(size.height) / 2.0,
                )))
            });

        let Some(location) = target else {
            return false;
        };

        self.pointer_recenter_pending = false;

        let focus = {
            let scene = crate::render::RenderScene::from_window_manager(&self.window_manager);
            scene.surface_under(location).map(|(surface, origin, _)| {
                (
                    surface,
                    Point::from((f64::from(origin.x), f64::from(origin.y))),
                )
            })
        };

        let time = Clock::<Monotonic>::new().now().as_millis() as u32;
        pointer.motion(
            self,
            focus,
            &MotionEvent {
                location,
                serial: SERIAL_COUNTER.next_serial(),
                time,
            },
        );
        pointer.frame(self);
        // Output publication, hotplug and session restoration all converge here.
        // Restart the first-use cursor warm-up after the new output is ready so
        // an idle pointer still gets frames while its size/theme asset is loaded.
        self.begin_cursor_asset_warmup();
        self.request_all_output_frames();
        true
    }
    /// Applies a deferred output-driven recenter once the active client grab ends.
    ///
    /// Output removal must not break an XDG popup or another legitimate pointer
    /// grab. If recentering was deferred, the event loop retries after dispatch;
    /// the first iteration after Smithay releases the grab restores the pointer
    /// to an enabled output and schedules the corresponding frames.
    pub fn apply_deferred_pointer_recenter(&mut self) -> bool {
        if !self.pointer_recenter_pending {
            return false;
        }
        self.center_pointer_on_primary_output()
    }

    /// Re-evaluates pointer focus after a surface changes the visible scene.
    ///
    /// The pointer remains at its current logical location; this only refreshes
    /// the surface under it. Active client grabs are never disturbed.
    pub fn refresh_pointer_focus_at_current_location(&mut self) -> bool {
        let Some(pointer) = self.seat.get_pointer() else {
            return false;
        };

        let requested = pointer.current_location();
        let output_geometry = self
            .output_manager
            .registry()
            .iter()
            .filter(|output| output.is_enabled())
            .filter_map(|output| {
                let bounds = self
                    .output_manager
                    .logical_bounds(output.id())
                    .ok()
                    .flatten()?;
                let origin = bounds.origin();
                let size = bounds.size();

                super::pointer_bounds::OutputGeometry::new(
                    origin.x as f64,
                    origin.y as f64,
                    f64::from(size.width),
                    f64::from(size.height),
                )
            })
            .collect::<Vec<_>>();

        let Some(location) =
            super::pointer_bounds::constrain_to_outputs(requested, output_geometry.iter().copied())
        else {
            return false;
        };

        if pointer.is_grabbed() {
            if location != requested {
                self.pointer_recenter_pending = true;
            }
            return false;
        }

        let focus = {
            let scene = crate::render::RenderScene::from_window_manager(&self.window_manager);
            scene.surface_under(location).map(|(surface, origin, _)| {
                (
                    surface,
                    Point::from((f64::from(origin.x), f64::from(origin.y))),
                )
            })
        };

        let time = Clock::<Monotonic>::new().now().as_millis() as u32;
        pointer.motion(
            self,
            focus,
            &MotionEvent {
                location,
                serial: SERIAL_COUNTER.next_serial(),
                time,
            },
        );
        pointer.frame(self);
        self.request_all_output_frames();
        true
    }
}
