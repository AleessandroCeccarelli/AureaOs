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
    manager.register_device(device_path.clone()).unwrap();
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
    manager.register_device(primary.clone()).unwrap();
    manager.register_device(path("/dev/dri/card1")).unwrap();
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
    manager.register_device(first.clone()).unwrap();
    manager.register_device(path("/dev/dri/card1")).unwrap();
    assert_eq!(manager.selected_render_device().unwrap().path(), first);
}

#[test]
fn output_gpu_is_used_when_no_preference_or_primary_exists() {
    let mut manager = GpuManager::new();
    let output = path("/dev/dri/card1");
    manager.register_device(path("/dev/dri/card0")).unwrap();
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

#[test]
fn primary_policy_prefers_explicit_hardware_device() {
    let mut manager = GpuManager::new();
    let panel = path("/dev/dri/card0");
    let preferred = path("/dev/dri/card1");
    manager.register_device(panel.clone()).unwrap();
    manager.register_device(preferred.clone()).unwrap();
    manager
        .set_selection_metadata(
            &panel,
            GpuSelectionMetadata {
                hardware_rendering: true,
                has_builtin_panel: true,
                ..Default::default()
            },
        )
        .unwrap();
    manager
        .set_selection_metadata(
            &preferred,
            GpuSelectionMetadata {
                hardware_rendering: true,
                preferred_primary: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        manager.select_primary_by_policy().unwrap().path(),
        preferred
    );
}

#[test]
fn primary_policy_prefers_builtin_panel_before_platform_and_boot_vga() {
    let mut manager = GpuManager::new();
    let platform = path("/dev/dri/card0");
    let panel = path("/dev/dri/card1");
    manager.register_device(platform.clone()).unwrap();
    manager.register_device(panel.clone()).unwrap();
    manager
        .set_selection_metadata(
            &platform,
            GpuSelectionMetadata {
                hardware_rendering: true,
                is_platform_device: true,
                ..Default::default()
            },
        )
        .unwrap();
    manager
        .set_selection_metadata(
            &panel,
            GpuSelectionMetadata {
                hardware_rendering: true,
                has_builtin_panel: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(manager.select_primary_by_policy().unwrap().path(), panel);
}

#[test]
fn primary_policy_prefers_hardware_renderer_before_software_panel_candidate() {
    let mut manager = GpuManager::new();
    let software = path("/dev/dri/card0");
    let hardware = path("/dev/dri/card1");
    manager.register_device(software.clone()).unwrap();
    manager.register_device(hardware.clone()).unwrap();
    manager
        .set_selection_metadata(
            &software,
            GpuSelectionMetadata {
                has_builtin_panel: true,
                ..Default::default()
            },
        )
        .unwrap();
    manager
        .set_selection_metadata(
            &hardware,
            GpuSelectionMetadata {
                hardware_rendering: true,
                is_boot_vga: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(manager.select_primary_by_policy().unwrap().path(), hardware);
}

#[test]
fn primary_policy_honors_explicit_preference_without_hardware_renderer() {
    let mut manager = GpuManager::new();
    let preferred = path("/dev/dri/card0");
    let hardware = path("/dev/dri/card1");
    manager.register_device(preferred.clone()).unwrap();
    manager.register_device(hardware.clone()).unwrap();
    manager
        .set_selection_metadata(
            &preferred,
            GpuSelectionMetadata {
                preferred_primary: true,
                ..Default::default()
            },
        )
        .unwrap();
    manager
        .set_selection_metadata(
            &hardware,
            GpuSelectionMetadata {
                hardware_rendering: true,
                has_builtin_panel: true,
                ..Default::default()
            },
        )
        .unwrap();

    assert_eq!(
        manager.select_primary_by_policy().unwrap().path(),
        preferred
    );
}

#[test]
fn primary_policy_ignores_unavailable_devices() {
    let mut manager = GpuManager::new();
    let unavailable = path("/dev/dri/card0");
    let available = path("/dev/dri/card1");
    manager.register_device(unavailable.clone()).unwrap();
    manager.register_device(available.clone()).unwrap();
    manager
        .set_selection_metadata(
            &unavailable,
            GpuSelectionMetadata {
                preferred_primary: true,
                hardware_rendering: true,
                ..Default::default()
            },
        )
        .unwrap();
    manager
        .set_device_availability(&unavailable, GpuAvailability::Unavailable)
        .unwrap();
    manager
        .set_selection_metadata(
            &available,
            GpuSelectionMetadata {
                hardware_rendering: true,
                is_boot_vga: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        manager.select_primary_by_policy().unwrap().path(),
        available
    );
}


#[test]
fn output_selection_prefers_ready_hardware_over_preferred_software_renderer() {
    let mut manager = GpuManager::new();
    let software = path("/dev/dri/card0");
    let hardware = path("/dev/dri/card1");
    manager.register_device(software.clone()).unwrap();
    manager.register_device(hardware.clone()).unwrap();
    manager.set_preferred_render_device(&software).unwrap();
    manager
        .set_selection_metadata(
            &hardware,
            GpuSelectionMetadata {
                hardware_rendering: true,
                ..Default::default()
            },
        )
        .unwrap();

    let selected = manager
        .selected_render_device_for_output_if(&software, |device| {
            device.path() == software || device.path() == hardware
        })
        .unwrap();

    assert_eq!(selected.path(), hardware);
}

#[test]
fn output_selection_uses_software_only_when_no_hardware_renderer_is_ready() {
    let mut manager = GpuManager::new();
    let software = path("/dev/dri/card0");
    let hardware = path("/dev/dri/card1");
    manager.register_device(software.clone()).unwrap();
    manager.register_device(hardware.clone()).unwrap();
    manager.set_preferred_render_device(&software).unwrap();
    manager
        .set_selection_metadata(
            &hardware,
            GpuSelectionMetadata {
                hardware_rendering: true,
                ..Default::default()
            },
        )
        .unwrap();

    let selected = manager
        .selected_render_device_for_output_if(&software, |device| device.path() == software)
        .unwrap();

    assert_eq!(selected.path(), software);
}
