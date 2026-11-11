// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS test module relocated from production source.

use super::*;

fn valid_descriptor() -> DrmOutputDescriptor {
    let mode = DrmModeInfo::new(1920, 1080, 60_000);

    DrmOutputDescriptor::new(1, "HDMI-A-1", vec![mode], mode, 1)
}

#[test]
fn converts_valid_descriptor() {
    let output = valid_descriptor()
        .into_display_output()
        .expect("valid descriptor should convert");

    assert_eq!(output.id(), 1);
    assert_eq!(output.name(), "HDMI-A-1");
    assert_eq!(output.current_mode().width(), 1920);
    assert_eq!(output.current_mode().height(), 1080);
}

#[test]
fn rejects_invalid_mode() {
    let invalid = DrmModeInfo::new(0, 1080, 60_000);

    let descriptor = DrmOutputDescriptor::new(1, "HDMI-A-1", vec![invalid], invalid, 1);

    assert_eq!(
        descriptor.into_display_output(),
        Err(DrmOutputError::InvalidMode),
    );
}

#[test]
fn rejects_duplicate_modes() {
    let mode = DrmModeInfo::new(1920, 1080, 60_000);

    let descriptor = DrmOutputDescriptor::new(1, "HDMI-A-1", vec![mode, mode], mode, 1);

    assert_eq!(
        descriptor.into_display_output(),
        Err(DrmOutputError::DuplicateMode),
    );
}

#[test]
fn rejects_unsupported_current_mode() {
    let supported = DrmModeInfo::new(1920, 1080, 60_000);
    let unsupported = DrmModeInfo::new(1280, 720, 60_000);

    let descriptor = DrmOutputDescriptor::new(1, "HDMI-A-1", vec![supported], unsupported, 1);

    assert_eq!(
        descriptor.into_display_output(),
        Err(DrmOutputError::UnsupportedCurrentMode),
    );
}

#[test]
fn rejects_zero_scale() {
    let mode = DrmModeInfo::new(1920, 1080, 60_000);

    let descriptor = DrmOutputDescriptor::new(1, "HDMI-A-1", vec![mode], mode, 0);

    assert_eq!(
        descriptor.into_display_output(),
        Err(DrmOutputError::InvalidOutput),
    );
}

#[test]
fn logical_descriptor_does_not_claim_to_have_native_drm_configuration() {
    let descriptor = valid_descriptor();

    assert!(!descriptor.has_drm_configuration());
    assert!(descriptor.connector_handle().is_none());
    assert!(descriptor.native_mode().is_none());
}

#[test]
fn logical_conversion_keeps_descriptor_native_data_available() {
    let descriptor = valid_descriptor();
    let _output = descriptor.clone().into_display_output().unwrap();

    assert_eq!(descriptor, valid_descriptor());
    assert!(!descriptor.has_drm_configuration());
}
