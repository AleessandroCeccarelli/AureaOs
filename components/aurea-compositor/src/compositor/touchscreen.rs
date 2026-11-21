// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Route absolute touchscreen contacts to their mapped Wayland output.

use smithay::{
    backend::input::TouchSlot,
    input::touch::{DownEvent, MotionEvent, UpEvent},
    utils::{Point, SERIAL_COUNTER},
};

use super::AureaCompositor;
use crate::{input::pointing::touchscreen_transform::transform_touch_coordinates, output::OutputId};

impl AureaCompositor {
    /// Converts normalized device coordinates into global logical coordinates.
    ///
    /// A touchscreen is automatically mapped only on a single-output desktop.
    /// Multi-monitor desktops require an explicit physical device assignment.
    fn touchscreen_location(
        &self,
        device: aurea_desktop_api::input::InputDeviceId,
        x: f64,
        y: f64,
    ) -> Option<Point<f64, smithay::utils::Logical>> {
        if !x.is_finite() || !y.is_finite() {
            return None;
        }

        let output: OutputId = self.touchscreen_output(device)?;
        let bounds = self.output_manager.logical_bounds(output).ok()??;
        let origin = bounds.origin();
        let size = bounds.size();
        let transform = self.output_manager.registry().get(output)?.transform();

        if size.width == 0 || size.height == 0 {
            return None;
        }

        let (transformed_x, transformed_y) = transform_touch_coordinates(
            transform,
            x,
            y,
            f64::from(size.width),
            f64::from(size.height),
        )?;
        let local_x = transformed_x.clamp(0.0, f64::from(size.width - 1));
        let local_y = transformed_y.clamp(0.0, f64::from(size.height - 1));

        Some(Point::from((
            f64::from(origin.x) + local_x,
            f64::from(origin.y) + local_y,
        )))
    }

    fn touchscreen_focus(
        &self,
        location: Point<f64, smithay::utils::Logical>,
    ) -> Option<(smithay::reexports::wayland_server::protocol::wl_surface::WlSurface, Point<f64, smithay::utils::Logical>)> {
        let scene = crate::render::RenderScene::from_window_manager(&self.window_manager);
        scene.surface_under(location).map(|(surface, origin, _)| {
            (
                surface,
                Point::from((f64::from(origin.x), f64::from(origin.y))),
            )
        })
    }

    /// Returns the logical pixel extent used to transform a device's absolute coordinates.
    pub fn touchscreen_extent(
        &self,
        device: aurea_desktop_api::input::InputDeviceId,
    ) -> Option<(i32, i32)> {
        let output = self.touchscreen_output(device)?;
        let bounds = self.output_manager.logical_bounds(output).ok()??;
        let size = bounds.size();
        let width = i32::try_from(size.width).ok()?;
        let height = i32::try_from(size.height).ok()?;

        (width > 0 && height > 0).then_some((width, height))
    }

    /// Starts a contact from one physical touchscreen.
    pub fn handle_touch_down(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        physical_slot: TouchSlot,
        x: f64,
        y: f64,
        time: u32,
    ) {
        let Some(location) = self.touchscreen_location(device, x, y) else {
            return;
        };
        let Some(slot) = self.touchscreen_slots.acquire(device, physical_slot) else {
            return;
        };
        let Some(touch) = self.touch.clone() else {
            self.touchscreen_slots.release(device, physical_slot);
            return;
        };

        touch.down(
            self,
            self.touchscreen_focus(location),
            &DownEvent {
                slot,
                location,
                serial: SERIAL_COUNTER.next_serial(),
                time,
            },
        );
        self.request_all_output_frames();
    }

    /// Updates an active contact without changing its original surface focus.
    pub fn handle_touch_motion(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        physical_slot: TouchSlot,
        x: f64,
        y: f64,
        time: u32,
    ) {
        let Some(location) = self.touchscreen_location(device, x, y) else {
            return;
        };
        let Some(slot) = self.touchscreen_slots.get(device, physical_slot) else {
            return;
        };
        let Some(touch) = self.touch.clone() else {
            return;
        };

        touch.motion(
            self,
            self.touchscreen_focus(location),
            &MotionEvent { slot, location, time },
        );
        self.request_all_output_frames();
    }

    /// Ends one contact and releases its seat-wide slot identifier.
    pub fn handle_touch_up(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        physical_slot: TouchSlot,
        time: u32,
    ) {
        let Some(slot) = self.touchscreen_slots.release(device, physical_slot) else {
            return;
        };
        let Some(touch) = self.touch.clone() else {
            return;
        };

        touch.up(
            self,
            &UpEvent {
                slot,
                serial: SERIAL_COUNTER.next_serial(),
                time,
            },
        );
        self.request_all_output_frames();
    }

    /// Ends all contacts from one device without cancelling other touchscreens.
    pub fn handle_touch_cancel(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        time: u32,
    ) {
        let slots = self.touchscreen_slots.remove_device(device);
        let Some(touch) = self.touch.clone() else {
            return;
        };

        for slot in slots {
            touch.up(
                self,
                &UpEvent {
                    slot,
                    serial: SERIAL_COUNTER.next_serial(),
                    time,
                },
            );
        }
        touch.frame(self);
        self.request_all_output_frames();
    }

    /// Delivers the end of a libinput touch frame to Wayland clients.
    pub fn handle_touch_frame(&mut self) {
        if let Some(touch) = self.touch.clone() {
            touch.frame(self);
            self.request_all_output_frames();
        }
    }
}
