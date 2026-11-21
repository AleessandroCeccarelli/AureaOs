// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

use aurea_desktop::{
    color_configuration_manager::DesktopColorConfigurationManager,
    display_color_assignment_store::DisplayColorAssignmentStore,
};
use aurea_desktop_api::display::{
    DisplayColorConfigurationAssignment, DisplayColorConfigurationError,
    DisplayColorConfigurationId, DisplayColorConfigurationManager, DisplayConnector,
    DisplayDescriptor, DisplayEvent, DisplayId,
};

mod tests {
    fn display(value: u128) -> DisplayId {
        DisplayId(value)
    }

    fn configuration(value: &str) -> DisplayColorConfigurationId {
        DisplayColorConfigurationId::new(value).unwrap()
    }

    #[test]
    fn persistent_assignments_survive_manager_recreation() {
        let path = std::env::temp_dir()
            .join(format!(
                "aurea-color-manager-{}-{}",
                std::process::id(),
                configuration("persisted").as_str()
            ))
            .join("assignments");
        let store = DisplayColorAssignmentStore::open(&path).unwrap();
        let output = display(55);
        let profile = configuration("persisted");

        {
            let manager = DesktopColorConfigurationManager::open_persistent(store.clone()).unwrap();
            manager.register_display(output).unwrap();
            manager.register_configuration(profile.clone()).unwrap();
            manager
                .assign(DisplayColorConfigurationAssignment {
                    display_id: output,
                    configuration_id: Some(profile.clone()),
                })
                .unwrap();
        }

        let restored = DesktopColorConfigurationManager::open_persistent(store).unwrap();
        assert_eq!(restored.assignments()[0].configuration_id, Some(profile));
        restored.register_display(output).unwrap();
        assert!(
            restored
                .assign(DisplayColorConfigurationAssignment {
                    display_id: output,
                    configuration_id: Some(configuration("persisted")),
                })
                .is_ok()
        );

        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn display_lifecycle_events_register_and_unregister_outputs() {
        let manager = DesktopColorConfigurationManager::new();
        let descriptor = DisplayDescriptor {
            id: display(12),
            name: "DP-1".to_owned(),
            connector: DisplayConnector::DisplayPort,
            connected: true,
            supported_modes: Vec::new(),
            current_mode: None,
            capabilities: Default::default(),
        };

        manager
            .handle_display_event(&DisplayEvent::Connected(descriptor.clone()))
            .unwrap();
        assert!(
            manager
                .assign(DisplayColorConfigurationAssignment {
                    display_id: descriptor.id,
                    configuration_id: None,
                })
                .is_ok()
        );

        manager
            .handle_display_event(&DisplayEvent::Disconnected(descriptor.id))
            .unwrap();
        assert_eq!(
            manager.assign(DisplayColorConfigurationAssignment {
                display_id: descriptor.id,
                configuration_id: None,
            }),
            Err(DisplayColorConfigurationError::DisplayNotFound)
        );
    }

    #[test]
    fn snapshot_reconciliation_does_not_discard_saved_assignments() {
        let manager = DesktopColorConfigurationManager::new();
        let output = display(21);
        let profile = configuration("calibrated");
        manager.register_configuration(profile.clone()).unwrap();
        manager.register_display(output).unwrap();
        manager
            .assign(DisplayColorConfigurationAssignment {
                display_id: output,
                configuration_id: Some(profile.clone()),
            })
            .unwrap();

        let snapshot = [DisplayDescriptor {
            id: display(22),
            name: "HDMI-A-1".to_owned(),
            connector: DisplayConnector::Hdmi,
            connected: true,
            supported_modes: Vec::new(),
            current_mode: None,
            capabilities: Default::default(),
        }];
        manager.synchronize_displays(&snapshot).unwrap();

        assert_eq!(manager.assignments()[0].configuration_id, Some(profile));
        assert_eq!(
            manager.assign(DisplayColorConfigurationAssignment {
                display_id: output,
                configuration_id: None,
            }),
            Err(DisplayColorConfigurationError::DisplayNotFound)
        );
    }

    #[test]
    fn assignments_are_independent_per_display() {
        let manager = DesktopColorConfigurationManager::new();
        let first = display(1);
        let second = display(2);
        let warm = configuration("warm");
        let neutral = configuration("neutral");

        manager.register_display(first).unwrap();
        manager.register_display(second).unwrap();
        manager.register_configuration(warm.clone()).unwrap();
        manager.register_configuration(neutral.clone()).unwrap();

        manager
            .assign(DisplayColorConfigurationAssignment {
                display_id: first,
                configuration_id: Some(warm.clone()),
            })
            .unwrap();
        manager
            .assign(DisplayColorConfigurationAssignment {
                display_id: second,
                configuration_id: Some(neutral.clone()),
            })
            .unwrap();

        let assignments = manager.assignments();
        assert_eq!(assignments.len(), 2);
        assert_eq!(
            assignments
                .iter()
                .find(|item| item.display_id == first)
                .unwrap()
                .configuration_id,
            Some(warm)
        );
        assert_eq!(
            assignments
                .iter()
                .find(|item| item.display_id == second)
                .unwrap()
                .configuration_id,
            Some(neutral)
        );
    }

    #[test]
    fn disconnected_display_keeps_assignment_for_reconnection() {
        let manager = DesktopColorConfigurationManager::new();
        let output = display(7);
        let profile = configuration("photo");
        manager.register_display(output).unwrap();
        manager.register_configuration(profile.clone()).unwrap();
        manager
            .assign(DisplayColorConfigurationAssignment {
                display_id: output,
                configuration_id: Some(profile.clone()),
            })
            .unwrap();

        manager.unregister_display(output).unwrap();
        assert_eq!(
            manager.assign(DisplayColorConfigurationAssignment {
                display_id: output,
                configuration_id: Some(profile.clone()),
            }),
            Err(DisplayColorConfigurationError::DisplayNotFound)
        );
        assert_eq!(manager.assignments().len(), 1);

        manager.register_display(output).unwrap();
        assert_eq!(manager.assignments()[0].configuration_id, Some(profile));
    }

    #[test]
    fn rejects_unknown_displays_and_configurations() {
        let manager = DesktopColorConfigurationManager::new();
        let output = display(3);
        manager.register_display(output).unwrap();

        assert_eq!(
            manager.assign(DisplayColorConfigurationAssignment {
                display_id: output,
                configuration_id: Some(configuration("missing")),
            }),
            Err(DisplayColorConfigurationError::ConfigurationNotFound)
        );
        assert_eq!(
            manager.assign(DisplayColorConfigurationAssignment {
                display_id: display(99),
                configuration_id: None,
            }),
            Err(DisplayColorConfigurationError::DisplayNotFound)
        );
    }

    #[test]
    fn referenced_configuration_cannot_be_removed() {
        let manager = DesktopColorConfigurationManager::new();
        let output = display(4);
        let profile = configuration("studio");
        manager.register_display(output).unwrap();
        manager.register_configuration(profile.clone()).unwrap();
        manager
            .assign(DisplayColorConfigurationAssignment {
                display_id: output,
                configuration_id: Some(profile.clone()),
            })
            .unwrap();

        assert_eq!(manager.unregister_configuration(&profile), Ok(false));
        manager.clear(output).unwrap();
        assert_eq!(manager.unregister_configuration(&profile), Ok(true));
    }
}
