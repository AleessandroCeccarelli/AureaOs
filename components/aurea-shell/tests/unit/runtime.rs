// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS test module relocated from production source.

use std::sync::mpsc;

use aurea_desktop_api::display::{DisplayColorAdjustment, DisplayColorController};

use super::ShellRuntime;
use crate::desktop::{NightLightMode, NightLightState};
use crate::error::ShellError;
use crate::event::ShellEvent;
use crate::event_bus::EventBus;
use crate::session::SessionState;
use crate::state::ShellState;


#[test]
fn compositor_workspace_updates_are_mirrored_exactly_by_shell() {
    use aurea_desktop_api::desktop_interaction::DesktopInteraction;

    let mut runtime = ShellRuntime::new();

    runtime.handle_desktop_interaction(DesktopInteraction::WorkspaceChanged { index: 2 });
    assert_eq!(runtime.desktop_state().workspaces.active(), 2);
    assert_eq!(
        runtime.take_desktop_interaction(),
        Some(DesktopInteraction::WorkspaceChanged { index: 2 })
    );

    runtime.handle_desktop_interaction(DesktopInteraction::WorkspaceChanged { index: 0 });
    assert_eq!(runtime.desktop_state().workspaces.active(), 0);
}

#[test]
fn shell_ignores_invalid_compositor_workspace_index() {
    use aurea_desktop_api::desktop_interaction::DesktopInteraction;
    use crate::desktop::WORKSPACE_COUNT;

    let mut runtime = ShellRuntime::new();
    runtime.handle_desktop_interaction(DesktopInteraction::WorkspaceChanged {
        index: WORKSPACE_COUNT,
    });

    assert_eq!(runtime.desktop_state().workspaces.active(), 0);
}

#[test]
fn overview_and_context_menu_interactions_are_preserved_for_presentation() {
    use aurea_desktop_api::desktop_interaction::DesktopInteraction;

    let mut runtime = ShellRuntime::new();
    runtime.handle_desktop_interaction(DesktopInteraction::ShowOverview);
    runtime.handle_desktop_interaction(DesktopInteraction::ShowContextMenu);

    assert_eq!(
        runtime.take_desktop_interaction(),
        Some(DesktopInteraction::ShowOverview)
    );
    assert_eq!(
        runtime.take_desktop_interaction(),
        Some(DesktopInteraction::ShowContextMenu)
    );
    assert_eq!(
        runtime.desktop_state().panels.active(),
        Some(crate::desktop::DesktopPanel::ContextMenu)
    );

    runtime.dismiss_desktop_panel();
    assert_eq!(runtime.desktop_state().panels.active(), None);
}

#[test]
fn runtime_starts_in_created_state() {
    let runtime = ShellRuntime::new();

    assert_eq!(runtime.state(), ShellState::Created);
    assert_eq!(runtime.session_state(), SessionState::Starting);
}

#[test]
fn runtime_starts_and_activates_session() {
    let mut runtime = ShellRuntime::new();

    runtime.start().unwrap();

    assert_eq!(runtime.state(), ShellState::Running);
    assert_eq!(runtime.session_state(), SessionState::Active);
}

#[test]
fn runtime_rejects_starting_twice_without_changing_state() {
    let mut runtime = ShellRuntime::new();

    runtime.start().unwrap();

    assert!(runtime.start().is_err());
    assert_eq!(runtime.state(), ShellState::Running);
    assert_eq!(runtime.session_state(), SessionState::Active);
}

#[test]
fn runtime_rejects_stop_before_start_without_changing_state() {
    let mut runtime = ShellRuntime::new();

    assert!(runtime.stop().is_err());
    assert_eq!(runtime.state(), ShellState::Created);
    assert_eq!(runtime.session_state(), SessionState::Starting);
}

#[test]
fn runtime_locks_and_unlocks_session() {
    let mut runtime = ShellRuntime::new();

    runtime.start().unwrap();

    runtime.handle_event(ShellEvent::LockRequested).unwrap();
    assert_eq!(runtime.session_state(), SessionState::Locked);

    runtime.handle_event(ShellEvent::UnlockRequested).unwrap();
    assert_eq!(runtime.session_state(), SessionState::Active);
}

#[test]
fn runtime_rejects_locking_an_already_locked_session() {
    let mut runtime = ShellRuntime::new();

    runtime.start().unwrap();
    runtime.handle_event(ShellEvent::LockRequested).unwrap();

    assert!(runtime.handle_event(ShellEvent::LockRequested).is_err());

    assert_eq!(runtime.session_state(), SessionState::Locked);
    assert_eq!(runtime.state(), ShellState::Running);
}

#[test]
fn runtime_rejects_unlocking_an_active_session() {
    let mut runtime = ShellRuntime::new();

    runtime.start().unwrap();

    assert!(runtime.handle_event(ShellEvent::UnlockRequested).is_err());

    assert_eq!(runtime.session_state(), SessionState::Active);
    assert_eq!(runtime.state(), ShellState::Running);
}

#[test]
fn runtime_shutdown_event_stops_session_and_runtime() {
    let mut runtime = ShellRuntime::new();

    runtime.start().unwrap();
    runtime.handle_event(ShellEvent::ShutdownRequested).unwrap();

    assert_eq!(runtime.session_state(), SessionState::Stopped);
    assert_eq!(runtime.state(), ShellState::Stopped);
}

#[test]
fn runtime_rejects_second_shutdown() {
    let mut runtime = ShellRuntime::new();

    runtime.start().unwrap();
    runtime.handle_event(ShellEvent::ShutdownRequested).unwrap();

    assert!(runtime.handle_event(ShellEvent::ShutdownRequested).is_err());

    assert_eq!(runtime.session_state(), SessionState::Stopped);
    assert_eq!(runtime.state(), ShellState::Stopped);
}

#[test]
fn runtime_event_loop_stops_on_shutdown_event() {
    let (event_bus, receiver) = EventBus::new();
    let mut runtime = ShellRuntime::new();

    runtime.start().unwrap();
    event_bus.publish(ShellEvent::ShutdownRequested).unwrap();

    runtime.run_event_loop(receiver).unwrap();

    assert_eq!(runtime.state(), ShellState::Stopped);
    assert_eq!(runtime.session_state(), SessionState::Stopped);
}

#[test]
fn runtime_event_loop_stops_when_channel_closes() {
    let (sender, receiver) = mpsc::channel();
    let mut runtime = ShellRuntime::new();

    runtime.start().unwrap();
    drop(sender);

    runtime.run_event_loop(receiver).unwrap();

    assert_eq!(runtime.state(), ShellState::Stopped);
    assert_eq!(runtime.session_state(), SessionState::Stopped);
}

#[test]
fn runtime_event_loop_processes_events_in_order() {
    let (sender, receiver) = mpsc::channel();
    let mut runtime = ShellRuntime::new();

    runtime.start().unwrap();

    sender.send(ShellEvent::LockRequested).unwrap();
    sender.send(ShellEvent::UnlockRequested).unwrap();
    sender.send(ShellEvent::ShutdownRequested).unwrap();

    runtime.run_event_loop(receiver).unwrap();

    assert_eq!(runtime.state(), ShellState::Stopped);
    assert_eq!(runtime.session_state(), SessionState::Stopped);
}

#[test]
fn runtime_default_creates_unstarted_runtime() {
    let runtime = ShellRuntime::default();

    assert_eq!(runtime.state(), ShellState::Created);
    assert_eq!(runtime.session_state(), SessionState::Starting);
}

#[test]
fn runtime_start_error_is_lifecycle_transition_error() {
    let mut runtime = ShellRuntime::new();

    runtime.start().unwrap();

    assert!(matches!(
        runtime.start(),
        Err(ShellError::InvalidLifecycleTransition(
            "Shell can only start from the Created state"
        ))
    ));
}

#[test]
fn runtime_publishes_manual_night_light_to_shared_display_channel() {
    let display_color = DisplayColorController::new();
    let compositor_view = display_color.clone();
    let mut runtime = ShellRuntime::with_display_color(display_color);

    let mut night_light = NightLightState::default();
    night_light.set_enabled(true);
    night_light.set_strength_percent(74);
    runtime.set_night_light(night_light, false);

    assert_eq!(compositor_view.adjustment().warmth_percent(), 74);
    assert_eq!(runtime.display_color_adjustment().warmth_percent(), 74);
}

#[test]
fn runtime_publishes_neutral_output_when_night_light_is_disabled() {
    let display_color = DisplayColorController::new();
    let compositor_view = display_color.clone();
    let mut runtime = ShellRuntime::with_display_color(display_color);

    let mut night_light = NightLightState::default();
    night_light.set_enabled(true);
    night_light.set_strength_percent(80);
    runtime.set_night_light(night_light, true);
    assert_eq!(compositor_view.adjustment().warmth_percent(), 80);

    night_light.set_enabled(false);
    runtime.set_night_light(night_light, true);

    assert_eq!(
        compositor_view.adjustment(),
        DisplayColorAdjustment::default()
    );
}

#[test]
fn runtime_respects_scheduled_night_light_activation() {
    let display_color = DisplayColorController::new();
    let compositor_view = display_color.clone();
    let mut runtime = ShellRuntime::with_display_color(display_color);

    let mut night_light = NightLightState::default();
    night_light.set_enabled(true);
    night_light.set_mode(NightLightMode::CustomSchedule);
    runtime.set_night_light(night_light, false);
    assert!(compositor_view.adjustment().is_neutral());

    runtime.update_night_light(true);
    assert_eq!(compositor_view.adjustment().warmth_percent(), 50);
}
