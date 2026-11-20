mod tests {
    use crate::gpu::*;
    use std::path::PathBuf;

    fn path(value: &str) -> PathBuf {
        PathBuf::from(value)
    }

    #[test]
    fn manager_starts_empty() {
        let manager = GpuManager::new();

        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
        assert!(manager.selected_render_device().is_none());
    }

    #[test]
    fn registers_available_device() {
        let mut manager = GpuManager::new();
        let device_path = path("/dev/dri/card0");

        assert!(manager.register_device(device_path.clone()).is_ok());

        let device = manager.get_device(&device_path).unwrap();

        assert!(device.is_available());
        assert_eq!(device.path(), device_path);
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn rejects_relative_device_path() {
        let mut manager = GpuManager::new();

        assert_eq!(
            manager.register_device("dev/dri/card0"),
            Err(GpuManagerError::InvalidDevicePath(path("dev/dri/card0")))
        );

        assert!(manager.is_empty());
    }

    #[test]
    fn rejects_duplicate_device() {
        let mut manager = GpuManager::new();
        let device_path = path("/dev/dri/card0");

        manager.register_device(device_path.clone()).unwrap();

        assert_eq!(
            manager.register_device(device_path.clone()),
            Err(GpuManagerError::DeviceAlreadyRegistered(device_path))
        );

        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn rejects_unknown_primary_device() {
        let mut manager = GpuManager::new();
        let device_path = path("/dev/dri/card0");

        assert_eq!(
            manager.set_primary_device(&device_path),
            Err(GpuManagerError::UnknownDevice(device_path))
        );
    }

    #[test]
    fn selects_primary_device_by_default() {
        let mut manager = GpuManager::new();
        let primary = path("/dev/dri/card0");
        let secondary = path("/dev/dri/card1");

        manager.register_device(primary.clone()).unwrap();
        manager.register_device(secondary).unwrap();
        manager.set_primary_device(&primary).unwrap();

        assert_eq!(manager.selected_render_device().unwrap().path(), primary);
    }

    #[test]
    fn preferred_device_overrides_primary() {
        let mut manager = GpuManager::new();
        let primary = path("/dev/dri/card0");
        let preferred = path("/dev/dri/card1");

        manager.register_device(primary.clone()).unwrap();
        manager.register_device(preferred.clone()).unwrap();

        manager.set_primary_device(&primary).unwrap();
        manager.set_preferred_render_device(&preferred).unwrap();

        assert_eq!(manager.selected_render_device().unwrap().path(), preferred);
    }

    #[test]
    fn unavailable_preferred_device_falls_back_to_primary() {
        let mut manager = GpuManager::new();
        let primary = path("/dev/dri/card0");
        let preferred = path("/dev/dri/card1");

        manager.register_device(primary.clone()).unwrap();
        manager.register_device(preferred.clone()).unwrap();

        manager.set_primary_device(&primary).unwrap();
        manager.set_preferred_render_device(&preferred).unwrap();

        manager
            .set_device_availability(&preferred, GpuAvailability::Unavailable)
            .unwrap();

        assert_eq!(manager.selected_render_device().unwrap().path(), primary);
    }

    #[test]
    fn falls_back_to_first_available_device_without_primary() {
        let mut manager = GpuManager::new();
        let first = path("/dev/dri/card0");
        let second = path("/dev/dri/card1");

        manager.register_device(first.clone()).unwrap();
        manager.register_device(second.clone()).unwrap();

        assert_eq!(manager.selected_render_device().unwrap().path(), first);
    }

    #[test]
    fn output_gpu_is_used_when_no_preference_or_primary_exists() {
        let mut manager = GpuManager::new();
        let unrelated = path("/dev/dri/card0");
        let output = path("/dev/dri/card1");

        manager.register_device(unrelated).unwrap();
        manager.register_device(output.clone()).unwrap();

        assert_eq!(
            manager
                .selected_render_device_for_output(&output)
                .unwrap()
                .path(),
            output
        );
    }

    #[test]
    fn preferred_gpu_overrides_output_gpu_for_output_selection() {
        let mut manager = GpuManager::new();
        let output = path("/dev/dri/card0");
        let preferred = path("/dev/dri/card1");

        manager.register_device(output.clone()).unwrap();
        manager.register_device(preferred.clone()).unwrap();
        manager.set_preferred_render_device(&preferred).unwrap();

        assert_eq!(
            manager
                .selected_render_device_for_output(&output)
                .unwrap()
                .path(),
            preferred
        );
    }

    #[test]
    fn removing_primary_clears_primary_and_preference() {
        let mut manager = GpuManager::new();
        let device_path = path("/dev/dri/card0");

        manager.register_device(device_path.clone()).unwrap();
        manager.set_primary_device(&device_path).unwrap();
        manager.set_preferred_render_device(&device_path).unwrap();

        assert!(manager.remove_device(&device_path).is_some());
        assert!(manager.primary_device().is_none());
        assert!(manager.preferred_render_device().is_none());
        assert!(manager.selected_render_device().is_none());
    }

    #[test]
    fn availability_update_rejects_unknown_device() {
        let mut manager = GpuManager::new();
        let device_path = path("/dev/dri/card9");

        assert_eq!(
            manager.set_device_availability(&device_path, GpuAvailability::Unavailable),
            Err(GpuManagerError::UnknownDevice(device_path))
        );
    }

    #[test]
    fn clearing_preference_restores_primary_selection() {
        let mut manager = GpuManager::new();
        let primary = path("/dev/dri/card0");
        let preferred = path("/dev/dri/card1");

        manager.register_device(primary.clone()).unwrap();
        manager.register_device(preferred.clone()).unwrap();

        manager.set_primary_device(&primary).unwrap();
        manager.set_preferred_render_device(&preferred).unwrap();

        manager.clear_preferred_render_device();

        assert_eq!(manager.selected_render_device().unwrap().path(), primary);
    }
}
