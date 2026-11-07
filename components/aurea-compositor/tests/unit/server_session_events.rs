// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS test module relocated from production source.

use super::*;

#[test]
fn pause_event_updates_session_state() {
    let mut active = true;

    let mut devices = crate::drm::device_manager::DrmDeviceManager::new();
    let result = apply_session_event(SessionEvent::PauseSession, &mut active, &mut devices);

    assert!(result.is_ok());
    assert!(!active);
}

#[test]
fn activate_event_updates_session_state() {
    let mut active = false;

    let mut devices = crate::drm::device_manager::DrmDeviceManager::new();
    let result = apply_session_event(SessionEvent::ActivateSession, &mut active, &mut devices);

    assert!(result.is_ok());
    assert!(active);
}
