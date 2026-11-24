// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Pointer capability initialization and relative motion routing.

use std::{
    io,
    sync::atomic::{AtomicU64, Ordering},
};

static POINTER_POSITION_LOG_COUNT: AtomicU64 = AtomicU64::new(0);

use smithay::{
    input::pointer::{MotionEvent, PointerHandle},
    utils::{Point, SERIAL_COUNTER},
};

use super::{AureaCompositor, pointer_bounds::OutputGeometry};

impl AureaCompositor {
    /// Initializes the seat's Wayland pointer capability.
    pub fn initialize_pointer(&mut self) -> io::Result<()> {
        if self.seat.get_pointer().is_none() {
            let _pointer: PointerHandle<Self> = self.seat.add_pointer();
        }

        if self.seat.get_pointer().is_none() {
            return Err(io::Error::other(
                "Smithay did not retain the Wayland seat pointer capability",
            ));
        }

        tracing::info!("AureaOS Wayland pointer capability initialized");
        Ok(())
    }

    /// Routes relative device motion to the topmost client surface.
    ///
    /// Coordinates are maintained in global logical compositor space. Motion
    /// is constrained to the union of enabled output rectangles so the pointer
    /// cannot become stranded in desktop gaps or beyond a display edge.
    /// Absolute devices are handled separately because they require
    /// output-local mapping.
    pub fn handle_pointer_motion(
        &mut self,
        delta: Point<f64, smithay::utils::Logical>,
        unaccelerated_delta: Point<f64, smithay::utils::Logical>,
        time: u32,
    ) {
        if self.pointer_settings.shake_to_locate()
            && self.pointer_shake.update(unaccelerated_delta, time)
        {
            self.cursor_locate_animation.trigger();
        }

        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };

        let current = pointer.current_location();
        let requested = Point::from((current.x + delta.x, current.y + delta.y));
        let Some(location) = super::pointer_bounds::constrain_to_outputs(
            requested,
            self.output_manager
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

                    OutputGeometry::new(
                        origin.x as f64,
                        origin.y as f64,
                        f64::from(size.width),
                        f64::from(size.height),
                    )
                }),
        ) else {
            return;
        };

        let focus = {
            let scene = crate::render::RenderScene::from_window_manager(&self.window_manager);
            scene.surface_under(location).map(|(surface, origin, _)| {
                (
                    surface,
                    Point::from((f64::from(origin.x), f64::from(origin.y))),
                )
            })
        };

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
        let count = POINTER_POSITION_LOG_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
        if count == 1 || count % 120 == 0 {
            let actual = pointer.current_location();
            eprintln!(
                "AureaOS pointer: motion={} requested=({:.2},{:.2}) applied=({:.2},{:.2})",
                count, requested.x, requested.y, actual.x, actual.y
            );
        }
        self.request_all_output_frames();
    }
}
