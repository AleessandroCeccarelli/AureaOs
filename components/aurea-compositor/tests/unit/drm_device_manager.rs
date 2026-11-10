// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Tests for DRM device manager registration and lifecycle.

use aurea_compositor::drm::device_manager::*;
use aurea_compositor::drm::device::DrmDeviceError;

    use super::*;

    #[derive(Debug)]
    struct FakeDevice {
        node: u32,
        visits: usize,
    }

    impl ManagedDrmDevice for FakeDevice {
        type Identity = u32;
        fn identity(&self) -> Result<Self::Identity, DrmDeviceManagerError> {
            Ok(self.node)
        }
        fn pause(&mut self) {
            self.visits += 1;
        }
        fn activate(&mut self) -> Result<(), DrmDeviceError> {
            self.visits += 1;
            Ok(())
        }
    }

    fn node(minor: u32) -> u32 {
        minor
    }

    fn device(minor: u32) -> FakeDevice {
        FakeDevice {
            node: node(minor),
            visits: 0,
        }
    }

    #[test]
    fn manager_starts_empty() {
        let manager = DrmDeviceManager::<DrmDevice>::new();

        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
        assert_eq!(manager.iter().count(), 0);
    }

    #[test]
    fn manager_can_be_created_with_default() {
        let manager = DrmDeviceManager::<DrmDevice>::default();

        assert!(manager.is_empty());
    }

    #[test]
    fn registers_and_looks_up_distinct_devices() {
        let mut manager = DrmDeviceManager::<FakeDevice, u32>::new();
        assert_eq!(manager.register(device(1)).unwrap(), node(1));
        assert_eq!(manager.register(device(2)).unwrap(), node(2));

        assert_eq!(manager.len(), 2);
        assert!(manager.get(node(1)).is_some());
        assert!(manager.get(node(2)).is_some());
        assert!(manager.get(node(3)).is_none());
    }

    #[test]
    fn duplicate_registration_preserves_existing_device_and_manager() {
        let mut manager = DrmDeviceManager::<FakeDevice, u32>::new();
        manager.register(device(1)).unwrap();

        assert_eq!(
            manager.register(device(1)),
            Err(DrmDeviceManagerError::DeviceAlreadyRegistered(
                "1".to_string()
            ))
        );
        assert_eq!(manager.len(), 1);
        assert_eq!(manager.get(node(1)).map(|device| device.visits), Some(0));
    }

    #[test]
    fn removal_only_removes_the_requested_device() {
        let mut manager = DrmDeviceManager::<FakeDevice, u32>::new();
        manager.register(device(1)).unwrap();
        manager.register(device(2)).unwrap();

        let removed = manager.remove(node(1));
        assert_eq!(removed.map(|device| device.node), Some(node(1)));
        assert!(!manager.contains(node(1)));
        assert!(manager.contains(node(2)));
        assert!(manager.remove(node(3)).is_none());
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn iteration_visits_and_mutates_every_device() {
        let mut manager = DrmDeviceManager::<FakeDevice, u32>::new();
        manager.register(device(1)).unwrap();
        manager.register(device(2)).unwrap();

        let visited = manager
            .iter()
            .map(|(node, _)| *node)
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(visited, [node(1), node(2)].into_iter().collect());

        for (_, device) in manager.iter_mut() {
            device.pause();
        }
        assert!(manager.iter().all(|(_, device)| device.visits == 1));
    }
