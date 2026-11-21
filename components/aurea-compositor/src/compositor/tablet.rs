// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Routes libinput tablet-tool events to Smithay's Wayland tablet seat.

use smithay::{
    backend::{
        input::{
            AbsolutePositionEvent, Event, TabletToolAxisEvent,
            TabletToolButtonEvent, TabletToolEvent, TabletToolProximityEvent,
            TabletToolTipEvent,
        },
        libinput::LibinputInputBackend,
    },
    input::tablet::{
        TabletDescriptor, TabletSeatTrait,
        tool::{
            AxisFrame, ButtonEvent, DownEvent, MotionEvent, ProximityInEvent,
            ProximityOutEvent, UpEvent,
        },
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Point, SERIAL_COUNTER},
};

use super::AureaCompositor;
use crate::input::pointing::{id_for, TabletOutputTarget};

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
        let outputs = self.output_manager.registry().iter()
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

    fn tablet_location<E>(
        &self,
        event: &E,
    ) -> Option<Point<f64, smithay::utils::Logical>>
    where
        E: AbsolutePositionEvent<LibinputInputBackend> + Event<LibinputInputBackend>,
    {
        let device_id = id_for(&event.device());
        let (origin, width, height) = self.tablet_area(device_id)?;
        let x = event.x_transformed(width).clamp(0.0, f64::from(width - 1));
        let y = event.y_transformed(height).clamp(0.0, f64::from(height - 1));

        Some(Point::from((
            origin.x as f64 + x,
            origin.y as f64 + y,
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

    fn tablet_axis_frame<E>(event: &E) -> Option<AxisFrame>
    where
        E: TabletToolEvent<LibinputInputBackend>,
    {
        let mut frame = AxisFrame::new();
        let mut changed = false;

        if event.pressure_has_changed() {
            frame = frame.pressure(event.pressure());
            changed = true;
        }
        if event.distance_has_changed() {
            frame = frame.distance(event.distance());
            changed = true;
        }
        if event.tilt_has_changed() {
            let (x, y) = event.tilt();
            frame = frame.tilt(x, y);
            changed = true;
        }
        if event.rotation_has_changed() {
            frame = frame.rotation(event.rotation());
            changed = true;
        }
        if event.slider_has_changed() {
            frame = frame.slider(event.slider_position());
            changed = true;
        }
        if event.wheel_has_changed() {
            frame = frame.wheel(event.wheel_delta(), event.wheel_delta_discrete());
            changed = true;
        }

        changed.then_some(frame)
    }

    pub fn handle_tablet_proximity<E>(&mut self, event: E)
    where
        E: TabletToolProximityEvent<LibinputInputBackend>,
    {
        let tool_descriptor = event.tool();
        let tablet_descriptor = TabletDescriptor::from(&event.device());
        let tablet_seat = self.seat.tablet_seat();

        match event.state() {
            smithay::backend::input::ProximityState::In => {
                let Some(location) = self.tablet_location(&event) else {
                    return;
                };
                let tablet = tablet_seat
                    .get_tablet(&tablet_descriptor)
                    .unwrap_or_else(|| tablet_seat.add_tablet(&tablet_descriptor));
                let tool = tablet_seat
                    .get_tool(&tool_descriptor)
                    .unwrap_or_else(|| tablet_seat.add_tool(&tool_descriptor));
                let proximity = ProximityInEvent {
                    location,
                    axis: Self::tablet_axis_frame(&event),
                    serial: SERIAL_COUNTER.next_serial(),
                    time: event.time(),
                };
                tool.proximity_in(self, self.tablet_focus(location), tablet, &proximity);
                tool.frame(self, event.time());
            }
            smithay::backend::input::ProximityState::Out => {
                let Some(tool) = tablet_seat.get_tool(&tool_descriptor) else {
                    return;
                };
                tool.proximity_out(
                    self,
                    &ProximityOutEvent {
                        serial: SERIAL_COUNTER.next_serial(),
                        time: event.time(),
                    },
                );
                tool.frame(self, event.time());
            }
        }
        self.request_all_output_frames();
    }

    pub fn handle_tablet_axis<E>(&mut self, event: E)
    where
        E: TabletToolAxisEvent<LibinputInputBackend>,
    {
        let tablet_seat = self.seat.tablet_seat();
        let descriptor = event.tool();
        let Some(tool) = tablet_seat.get_tool(&descriptor) else {
            return;
        };
        let Some(location) = self.tablet_location(&event) else {
            return;
        };

        if let Some(axis) = Self::tablet_axis_frame(&event) {
            tool.axis(self, axis);
        }
        if event.delta_x() != 0.0 || event.delta_y() != 0.0 {
            tool.motion(
                self,
                self.tablet_focus(location),
                &MotionEvent {
                    location,
                    serial: SERIAL_COUNTER.next_serial(),
                    time: event.time(),
                },
            );
        }
        tool.frame(self, event.time());
        self.request_all_output_frames();
    }

    pub fn handle_tablet_tip<E>(&mut self, event: E)
    where
        E: TabletToolTipEvent<LibinputInputBackend>,
    {
        let tablet_seat = self.seat.tablet_seat();
        let descriptor = event.tool();
        let Some(tool) = tablet_seat.get_tool(&descriptor) else {
            return;
        };

        if let Some(axis) = Self::tablet_axis_frame(&event) {
            tool.axis(self, axis);
        }

        let serial = SERIAL_COUNTER.next_serial();
        let time = event.time();
        match event.tip_state() {
            smithay::backend::input::TabletToolTipState::Down => {
                tool.down(self, &DownEvent { serial, time });
            }
            smithay::backend::input::TabletToolTipState::Up => {
                tool.up(self, &UpEvent { serial, time });
            }
        }
        tool.frame(self, time);
        self.request_all_output_frames();
    }

    pub fn handle_tablet_button<E>(&mut self, event: E)
    where
        E: TabletToolButtonEvent<LibinputInputBackend>,
    {
        let tablet_seat = self.seat.tablet_seat();
        let descriptor = event.tool();
        let Some(tool) = tablet_seat.get_tool(&descriptor) else {
            return;
        };

        let state = event.button_state();
        let time = event.time();
        tool.button(
            self,
            &ButtonEvent {
                serial: SERIAL_COUNTER.next_serial(),
                button: event.button(),
                state,
                time,
            },
        );
        tool.frame(self, time);
        self.request_all_output_frames();
    }
}
