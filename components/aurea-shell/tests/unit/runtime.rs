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
    use crate::desktop::WORKSPACE_COUNT;
    use aurea_desktop_api::desktop_interaction::DesktopInteraction;

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
    let position = aurea_desktop_api::desktop_interaction::DesktopPosition { x: 640, y: 360 };
    runtime.handle_desktop_interaction(DesktopInteraction::ShowContextMenu { position });

    assert_eq!(
        runtime.take_desktop_interaction(),
        Some(DesktopInteraction::ShowOverview)
    );
    assert_eq!(
        runtime.take_desktop_interaction(),
        Some(DesktopInteraction::ShowContextMenu { position })
    );
    assert_eq!(
        runtime.desktop_state().panels.active(),
        Some(crate::desktop::DesktopPanel::ContextMenu)
    );
    assert_eq!(runtime.desktop_state().panels.context_menu_position(), Some(position));

    runtime.dismiss_desktop_panel();
    assert_eq!(runtime.desktop_state().panels.active(), None);
}

#[test]
fn context_menu_dismiss_interaction_closes_only_the_context_menu() {
    use aurea_desktop_api::desktop_interaction::DesktopInteraction;

    let mut runtime = ShellRuntime::new();
    let position = aurea_desktop_api::desktop_interaction::DesktopPosition { x: 120, y: 90 };
    runtime.handle_desktop_interaction(DesktopInteraction::ShowContextMenu { position });
    runtime.handle_desktop_interaction(DesktopInteraction::DismissContextMenu);

    assert_eq!(runtime.desktop_state().panels.active(), None);
    assert_eq!(runtime.desktop_state().panels.context_menu_position(), None);
    assert_eq!(
        runtime.take_desktop_interaction(),
        Some(DesktopInteraction::ShowContextMenu { position })
    );
    assert_eq!(
        runtime.take_desktop_interaction(),
        Some(DesktopInteraction::DismissContextMenu)
    );
}

#[test]
fn app_expose_interaction_activates_the_app_expose_panel() {
    use aurea_desktop_api::desktop_interaction::DesktopInteraction;

    let mut runtime = ShellRuntime::new();
    runtime.handle_desktop_interaction(DesktopInteraction::ShowAppExpose);

    assert_eq!(
        runtime.desktop_state().panels.active(),
        Some(crate::desktop::DesktopPanel::AppExpose)
    );
    assert_eq!(
        runtime.take_desktop_interaction(),
        Some(DesktopInteraction::ShowAppExpose)
    );
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

#[test]
fn runtime_event_loop_receives_compositor_interactions() {
    use aurea_desktop_api::desktop_interaction::DesktopInteraction;
    use aurea_desktop_api::desktop_interaction::desktop_interaction_channel;

    let (event_bus, event_receiver) = EventBus::new();
    let (interaction_sender, interaction_receiver) = desktop_interaction_channel();
    let mut runtime = ShellRuntime::new();

    runtime.start().unwrap();
    interaction_sender
        .send(DesktopInteraction::WorkspaceChanged { index: 3 })
        .unwrap();
    interaction_sender
        .send(DesktopInteraction::ShowOverview)
        .unwrap();
    interaction_sender
        .send(DesktopInteraction::ShowAppExpose)
        .unwrap();
    event_bus.publish(ShellEvent::ShutdownRequested).unwrap();

    runtime
        .run_event_loop_with_interactions(event_receiver, interaction_receiver)
        .unwrap();

    assert_eq!(runtime.desktop_state().workspaces.active(), 3);
    assert_eq!(
        runtime.desktop_state().panels.active(),
        Some(crate::desktop::DesktopPanel::AppExpose)
    );
    assert_eq!(
        runtime.take_desktop_interaction(),
        Some(DesktopInteraction::WorkspaceChanged { index: 3 })
    );
    assert_eq!(
        runtime.take_desktop_interaction(),
        Some(DesktopInteraction::ShowOverview)
    );
    assert_eq!(
        runtime.take_desktop_interaction(),
        Some(DesktopInteraction::ShowAppExpose)
    );
}

#[test]
fn compositor_desktop_item_selection_is_applied_to_shell_state() {
    use aurea_desktop_api::desktop_interaction::DesktopInteraction;

    let mut runtime = ShellRuntime::new();
    assert_eq!(runtime.desktop_state().selected_item_id(), None);

    runtime.handle_desktop_interaction(DesktopInteraction::DesktopItemSelected {
        id: "system:trash".to_owned(),
    });
    assert_eq!(runtime.desktop_state().selected_item_id(), Some("system:trash"));
    assert_eq!(
        runtime.desktop_selection.selected_item().as_deref(),
        Some("system:trash")
    );
    assert_eq!(
        runtime.take_desktop_interaction(),
        Some(DesktopInteraction::DesktopItemSelected {
            id: "system:trash".to_owned(),
        })
    );

    runtime.handle_desktop_interaction(DesktopInteraction::DesktopSelectionCleared);
    assert_eq!(runtime.desktop_state().selected_item_id(), None);
    assert_eq!(runtime.desktop_selection.selected_item(), None);
}


#[test]
fn desktop_item_activation_is_retained_as_a_semantic_action() {
    use aurea_desktop_api::desktop_interaction::DesktopInteraction;

    let mut runtime = ShellRuntime::new();
    runtime.handle_desktop_interaction(DesktopInteraction::DesktopItemActivated {
        id: "system:trash".to_owned(),
    });

    assert_eq!(
        runtime.take_desktop_interaction(),
        Some(DesktopInteraction::DesktopItemActivated {
            id: "system:trash".to_owned(),
        })
    );
}


#[test]
fn selected_context_menu_command_closes_menu_and_is_retained_for_service_dispatch() {
    use aurea_desktop_api::{
        desktop::DesktopContextMenuCommand,
        desktop_interaction::{DesktopInteraction, DesktopPosition},
    };

    let mut runtime = ShellRuntime::new();
    runtime.handle_desktop_interaction(DesktopInteraction::ShowContextMenu {
        position: DesktopPosition { x: 120, y: 90 },
    });
    runtime.handle_desktop_interaction(DesktopInteraction::ContextMenuCommandSelected {
        command: DesktopContextMenuCommand::UseAsWallpaper,
    });

    assert_eq!(runtime.desktop_state().panels.active(), None);
    assert_eq!(
        runtime.take_desktop_interaction(),
        Some(DesktopInteraction::ShowContextMenu {
            position: DesktopPosition { x: 120, y: 90 },
        })
    );
    assert_eq!(
        runtime.take_desktop_interaction(),
        Some(DesktopInteraction::ContextMenuCommandSelected {
            command: DesktopContextMenuCommand::UseAsWallpaper,
        })
    );
}

#[test]
fn runtime_forwards_mouse_preferences_to_compositor_channel() {
    use aurea_desktop_api::input::{
        InputDeviceId, PointerSettings, pointer_settings_channel, touchpad_settings_channel,
    };

    let (touchpad_controller, _touchpad_receiver) = touchpad_settings_channel();
    let (pointer_controller, pointer_receiver) = pointer_settings_channel();
    let (desktop_pointer_behavior, _desktop_pointer_behavior_receiver) = desktop_pointer_behavior_channel();
    let runtime = ShellRuntime::with_controllers(
        DisplayColorController::new(),
        touchpad_controller,
        pointer_controller,
        desktop_pointer_behavior,
    );
    let settings = PointerSettings::default();

    assert!(runtime.set_pointer_settings(InputDeviceId(23), settings));

    let command = pointer_receiver.try_recv().unwrap();
    assert_eq!(command.device, InputDeviceId(23));
    assert_eq!(command.settings, settings);
}
