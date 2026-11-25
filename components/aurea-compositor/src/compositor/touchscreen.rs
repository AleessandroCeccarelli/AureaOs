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
use crate::{
    input::pointing::touchscreen_transform::transform_touch_coordinates, output::OutputId,
};

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

        let swaps_dimensions = matches!(
            transform,
            smithay::utils::Transform::_90
                | smithay::utils::Transform::_270
                | smithay::utils::Transform::Flipped90
                | smithay::utils::Transform::Flipped270
        );
        let (source_width, source_height) = if swaps_dimensions {
            (size.height, size.width)
        } else {
            (size.width, size.height)
        };
        // libinput supplies coordinates in the source output's pixel extent.
        // Calibration operates in normalized device space before output rotation.
        let source_max_x = f64::from(source_width.saturating_sub(1).max(1));
        let source_max_y = f64::from(source_height.saturating_sub(1).max(1));
        let (normalized_x, normalized_y) = self
            .touchscreen_calibrations
            .get(device)
            .apply(x / source_max_x, y / source_max_y)?;
        let calibrated_x = normalized_x * source_max_x;
        let calibrated_y = normalized_y * source_max_y;

        let (transformed_x, transformed_y) = transform_touch_coordinates(
            transform,
            calibrated_x,
            calibrated_y,
            f64::from(source_width),
            f64::from(source_height),
        )?;
        let local_x = transformed_x.clamp(0.0, f64::from(size.width - 1));
        let local_y = transformed_y.clamp(0.0, f64::from(size.height - 1));

        Some(Point::from((
            origin.x as f64 + local_x,
            origin.y as f64 + local_y,
        )))
    }

    fn touchscreen_focus(
        &self,
        location: Point<f64, smithay::utils::Logical>,
    ) -> Option<(
        smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
        Point<f64, smithay::utils::Logical>,
    )> {
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
        let transform = self.output_manager.registry().get(output)?.transform();
        let swaps_dimensions = matches!(
            transform,
            smithay::utils::Transform::_90
                | smithay::utils::Transform::_270
                | smithay::utils::Transform::Flipped90
                | smithay::utils::Transform::Flipped270
        );
        let (source_width, source_height) = if swaps_dimensions {
            (size.height, size.width)
        } else {
            (size.width, size.height)
        };
        let width = i32::try_from(source_width).ok()?;
        let height = i32::try_from(source_height).ok()?;

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
            &MotionEvent {
                slot,
                location,
                time,
            },
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

    /// Cancels the seat-wide touch sequence after a device reports cancellation.
    ///
    /// Smithay 0.7 exposes seat-wide cancellation only. If the cancelling device
    /// owns an active contact, all seat contacts are cancelled and every physical
    /// slot mapping is cleared so other devices cannot retain stale contacts.
    pub fn handle_touch_cancel(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        _time: u32,
    ) {
        let cancelled_device_slots = self.touchscreen_slots.remove_device(device);
        if cancelled_device_slots.is_empty() {
            return;
        }

        self.touchscreen_slots.clear_all();

        if let Some(touch) = self.touch.clone() {
            touch.cancel(self);
            self.request_all_output_frames();
        }
    }

    /// Delivers the end of a libinput touch frame to Wayland clients.
    pub fn handle_touch_frame(&mut self) {
        if let Some(touch) = self.touch.clone() {
            touch.frame(self);
            self.request_all_output_frames();
        }
    }
}
