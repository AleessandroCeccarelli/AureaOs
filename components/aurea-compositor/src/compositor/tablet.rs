// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Routes libinput tablet-tool events to Smithay's Wayland tablet seat.

use smithay::{
    backend::{
        input::{
            AbsolutePositionEvent, Event, TabletToolAxisEvent, TabletToolButtonEvent,
            TabletToolEvent, TabletToolProximityEvent, TabletToolTipEvent,
        },
        libinput::LibinputInputBackend,
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Point, SERIAL_COUNTER},
    wayland::tablet_manager::{TabletDescriptor, TabletSeatTrait},
};

use super::AureaCompositor;
use crate::input::pointing::{TabletOutputTarget, id_for};

/// Converts libinput microsecond timestamps to wrapping Wayland millisecond timestamps.
fn wayland_time(microseconds: u64) -> u32 {
    (microseconds / 1_000) as u32
}

impl AureaCompositor {
    /// Drops the advertised tablet when its physical libinput device disappears.
    pub fn remove_tablet_device(&mut self, device: &smithay::reexports::input::Device) {
        let tablet_seat = self.seat.tablet_seat();
        tablet_seat.remove_tablet(&TabletDescriptor::from(device));
        if tablet_seat.count_tablets() == 0 {
            tablet_seat.clear_tools();
        }
    }

    fn tablet_area(
        &self,
        device: aurea_desktop_api::input::InputDeviceId,
    ) -> Option<(Point<i64, smithay::utils::Logical>, i32, i32)> {
        let target = self.tablet_output_target(device)?;
        let outputs = self
            .output_manager
            .registry()
            .iter()
            .filter(|output| output.is_enabled())
            .filter(|output| match target {
                TabletOutputTarget::All => true,
                TabletOutputTarget::Output(id) => output.id() == id,
            });

        let mut min_x = i64::MAX;
        let mut min_y = i64::MAX;
        let mut max_x = i64::MIN;
        let mut max_y = i64::MIN;
        let mut found = false;

        for output in outputs {
            let bounds = self.output_manager.logical_bounds(output.id()).ok()??;
            let origin = bounds.origin();
            let size = bounds.size();
            let right = origin.x.checked_add(i64::from(size.width))?;
            let bottom = origin.y.checked_add(i64::from(size.height))?;

            min_x = min_x.min(origin.x);
            min_y = min_y.min(origin.y);
            max_x = max_x.max(right);
            max_y = max_y.max(bottom);
            found = true;
        }

        if !found {
            return None;
        }

        let width = i32::try_from(max_x.checked_sub(min_x)?).ok()?;
        let height = i32::try_from(max_y.checked_sub(min_y)?).ok()?;
        if width <= 0 || height <= 0 {
            return None;
        }

        Some((Point::from((min_x, min_y)), width, height))
    }

    fn tablet_location<E>(&self, event: &E) -> Option<Point<f64, smithay::utils::Logical>>
    where
        E: AbsolutePositionEvent<LibinputInputBackend> + Event<LibinputInputBackend>,
    {
        let device_id = id_for(&event.device());
        let (origin, width, height) = self.tablet_area(device_id)?;
        let raw_x = event.x_transformed(width) / f64::from(width);
        let raw_y = event.y_transformed(height) / f64::from(height);
        let profile = self.tablet_calibration(device_id);
        let output_aspect = f64::from(width) / f64::from(height);
        let (x, y) = profile.apply(raw_x, raw_y, output_aspect)?;
        Some(Point::from((
            origin.x as f64 + x * f64::from(width - 1),
            origin.y as f64 + y * f64::from(height - 1),
        )))
    }

    fn tablet_focus(
        &self,
        location: Point<f64, smithay::utils::Logical>,
    ) -> Option<(WlSurface, Point<f64, smithay::utils::Logical>)> {
        let scene = crate::render::RenderScene::from_window_manager(&self.window_manager);
        scene.surface_under(location).map(|(surface, origin, _)| {
            (
                surface,
                Point::from((f64::from(origin.x), f64::from(origin.y))),
            )
        })
    }

    fn apply_tablet_axes<E>(tool: &smithay::wayland::tablet_manager::TabletToolHandle, event: &E)
    where
        E: TabletToolEvent<LibinputInputBackend>,
    {
        if event.pressure_has_changed() {
            tool.pressure(event.pressure());
        }
        if event.distance_has_changed() {
            tool.distance(event.distance());
        }
        if event.tilt_has_changed() {
            tool.tilt(event.tilt());
        }
        if event.rotation_has_changed() {
            tool.rotation(event.rotation());
        }
        if event.slider_has_changed() {
            tool.slider_position(event.slider_position());
        }
        if event.wheel_has_changed() {
            tool.wheel(event.wheel_delta(), event.wheel_delta_discrete());
        }
    }

    fn ensure_tablet(
        &mut self,
        descriptor: &TabletDescriptor,
    ) -> smithay::wayland::tablet_manager::TabletHandle {
        let tablet_seat = self.seat.tablet_seat();
        if let Some(tablet) = tablet_seat.get_tablet(descriptor) {
            return tablet;
        }
        let display_handle = self.display_handle.clone();
        tablet_seat.add_tablet::<Self>(&display_handle, descriptor)
    }

    fn ensure_tablet_tool(
        &mut self,
        descriptor: &smithay::backend::input::TabletToolDescriptor,
    ) -> smithay::wayland::tablet_manager::TabletToolHandle {
        let tablet_seat = self.seat.tablet_seat();
        if let Some(tool) = tablet_seat.get_tool(descriptor) {
            return tool;
        }
        let display_handle = self.display_handle.clone();
        tablet_seat.add_tool::<Self>(self, &display_handle, descriptor)
    }

    pub fn handle_tablet_proximity<E>(&mut self, event: E)
    where
        E: TabletToolProximityEvent<LibinputInputBackend>,
    {
        let tool_descriptor = event.tool();
        let tablet_descriptor = TabletDescriptor::from(&event.device());
        let tablet = self.ensure_tablet(&tablet_descriptor);
        let tool = self.ensure_tablet_tool(&tool_descriptor);
        let time = wayland_time(event.time());

        match event.state() {
            smithay::backend::input::ProximityState::In => {
                let Some(location) = self.tablet_location(&event) else {
                    return;
                };
                Self::apply_tablet_axes(&tool, &event);
                if let Some(focus) = self.tablet_focus(location) {
                    tool.proximity_in(location, focus, &tablet, SERIAL_COUNTER.next_serial(), time);
                }
            }
            smithay::backend::input::ProximityState::Out => {
                tool.proximity_out(time);
            }
        }
        self.request_all_output_frames();
    }

    pub fn handle_tablet_axis<E>(&mut self, event: E)
    where
        E: TabletToolAxisEvent<LibinputInputBackend>,
    {
        let tool_descriptor = event.tool();
        let tablet_descriptor = TabletDescriptor::from(&event.device());
        let tablet = self.ensure_tablet(&tablet_descriptor);
        let Some(location) = self.tablet_location(&event) else {
            return;
        };
        let Some(tool) = self.seat.tablet_seat().get_tool(&tool_descriptor) else {
            return;
        };

        Self::apply_tablet_axes(&tool, &event);
        tool.motion(
            location,
            self.tablet_focus(location),
            &tablet,
            SERIAL_COUNTER.next_serial(),
            wayland_time(event.time()),
        );
        self.request_all_output_frames();
    }

    pub fn handle_tablet_tip<E>(&mut self, event: E)
    where
        E: TabletToolTipEvent<LibinputInputBackend>,
    {
        let descriptor = event.tool();
        let Some(tool) = self.seat.tablet_seat().get_tool(&descriptor) else {
            return;
        };

        Self::apply_tablet_axes(&tool, &event);
        let time = wayland_time(event.time());
        match event.tip_state() {
            smithay::backend::input::TabletToolTipState::Down => {
                tool.tip_down(SERIAL_COUNTER.next_serial(), time);
            }
            smithay::backend::input::TabletToolTipState::Up => tool.tip_up(time),
        }
        self.request_all_output_frames();
    }

    pub fn handle_tablet_button<E>(&mut self, event: E)
    where
        E: TabletToolButtonEvent<LibinputInputBackend>,
    {
        let descriptor = event.tool();
        let Some(tool) = self.seat.tablet_seat().get_tool(&descriptor) else {
            return;
        };

        tool.button(
            event.button(),
            event.button_state(),
            SERIAL_COUNTER.next_serial(),
            wayland_time(event.time()),
        );
        self.request_all_output_frames();
    }
}
