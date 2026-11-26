// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Pointer capability initialization and relative motion routing.

use std::io;

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
        device: aurea_desktop_api::input::InputDeviceId,
        delta: Point<f64, smithay::utils::Logical>,
        unaccelerated_delta: Point<f64, smithay::utils::Logical>,
        time: u32,
    ) {
        // Preserve sub-pixel precision, but never allow invalid backend values
        // to poison the shared seat cursor position.
        if !delta.x.is_finite()
            || !delta.y.is_finite()
            || !unaccelerated_delta.x.is_finite()
            || !unaccelerated_delta.y.is_finite()
        {
            tracing::warn!(device_id = device.0, "Discarded non-finite pointer motion sample");
            return;
        }

        if self.pointer_settings_for(device).shake_to_locate() {
            if self.pointer_shake.update(device, unaccelerated_delta, time) {
                self.cursor_locate_animation.trigger();
            }
        } else {
            self.pointer_shake.remove(device);
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

        self.commit_pointer_location(location, time);
        let trail_sample_time = std::time::Instant::now();
        if self.cursor_locate_animation.is_active(trail_sample_time) {
            self.cursor_trail.record(location, trail_sample_time);
        }
    }

    /// Routes an absolute pointer device to its assigned enabled output.
    ///
    /// Absolute coordinates are output-local. A single enabled output may be
    /// selected automatically; multi-output layouts require an explicit
    /// per-device assignment so the compositor never guesses which monitor
    /// owns the device. Touchscreens and tablet tools keep their dedicated
    /// paths.
    pub fn handle_pointer_motion_absolute<E>(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        event: &E,
        time: u32,
    ) -> bool
    where
        E: smithay::backend::input::AbsolutePositionEvent<
                smithay::backend::libinput::LibinputInputBackend,
            > + smithay::backend::input::Event<
                smithay::backend::libinput::LibinputInputBackend,
            >,
    {
        let Some(output_id) = self.absolute_pointer_output(device) else {
            tracing::debug!(device_id = device.0, "Absolute pointer motion ignored: explicit output assignment required");
            return false;
        };
        let bounds = self.output_manager.logical_bounds(output_id).ok().flatten();
        let Some(bounds) = bounds else {
            return false;
        };

        let origin = bounds.origin();
        let size = bounds.size();
        if size.width <= 0 || size.height <= 0 {
            return false;
        }

        let width = size.width;
        let height = size.height;
        let (Ok(event_width), Ok(event_height)) =
            (i32::try_from(width), i32::try_from(height))
        else {
            return false;
        };
        let requested = Point::from((
            origin.x as f64 + event.x_transformed(event_width) / f64::from(width)
                * f64::from(width - 1),
            origin.y as f64 + event.y_transformed(event_height) / f64::from(height)
                * f64::from(height - 1),
        ));
        let Some(output_geometry) = OutputGeometry::new(
            origin.x as f64,
            origin.y as f64,
            f64::from(width),
            f64::from(height),
        ) else {
            return false;
        };
        let Some(location) =
            super::pointer_bounds::constrain_to_outputs(requested, [output_geometry])
        else {
            return false;
        };

        self.commit_pointer_location(location, time)
    }

    /// Commits a validated global pointer location to Smithay and schedules
    /// presentation on every enabled output.
    ///
    /// Relative and absolute devices share this final stage so focus updates,
    /// Wayland pointer frames and native cursor redraws cannot diverge.
    fn commit_pointer_location(
        &mut self,
        location: Point<f64, smithay::utils::Logical>,
        time: u32,
    ) -> bool {
        let Some(pointer) = self.seat.get_pointer() else {
            return false;
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
        self.request_all_output_frames();
        true
    }
}
