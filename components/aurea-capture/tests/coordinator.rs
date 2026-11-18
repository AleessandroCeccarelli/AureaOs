// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Capture Core tests.

use aurea_capture::{
    AudioCaptureMode, CaptureCoordinator, CaptureKind, CaptureRequest, CaptureSessionState,
    CaptureTarget, DisplayId,
};

fn recording_request() -> CaptureRequest {
    CaptureRequest::new(
        CaptureKind::Recording { audio: AudioCaptureMode::SystemAndMicrophone },
        CaptureTarget::Display(DisplayId(1)),
        true,
    )
}

#[test]
fn session_requires_authorization_before_capture() {
    let mut coordinator = CaptureCoordinator::new();
    let id = coordinator.begin(recording_request()).unwrap();

    assert_eq!(coordinator.session().unwrap().state(), CaptureSessionState::AwaitingAuthorization);
    coordinator.authorize(id).unwrap();
    assert_eq!(coordinator.session().unwrap().state(), CaptureSessionState::Ready);
}

#[test]
fn recording_can_pause_resume_and_finalize() {
    let mut coordinator = CaptureCoordinator::new();
    let id = coordinator.begin(recording_request()).unwrap();

    coordinator.authorize(id).unwrap();
    coordinator.start(id).unwrap();
    coordinator.pause(id).unwrap();
    assert_eq!(coordinator.session().unwrap().state(), CaptureSessionState::Paused);

    coordinator.resume(id).unwrap();
    coordinator.begin_finalization(id).unwrap();
    coordinator.complete(id).unwrap();

    assert_eq!(coordinator.session().unwrap().state(), CaptureSessionState::Completed);
}

#[test]
fn screenshot_cannot_be_paused() {
    let mut coordinator = CaptureCoordinator::new();
    let request = CaptureRequest::new(
        CaptureKind::Screenshot,
        CaptureTarget::ActiveWindow,
        false,
    );
    let id = coordinator.begin(request).unwrap();

    coordinator.authorize(id).unwrap();
    coordinator.start(id).unwrap();

    assert!(coordinator.pause(id).is_err());
}

#[test]
fn only_one_non_terminal_session_is_allowed() {
    let mut coordinator = CaptureCoordinator::new();
    let first = coordinator.begin(recording_request()).unwrap();

    assert!(coordinator.begin(recording_request()).is_err());

    coordinator.cancel(first).unwrap();
    assert!(coordinator.begin(recording_request()).is_ok());
}

#[test]
fn cancelled_session_cannot_be_restarted() {
    let mut coordinator = CaptureCoordinator::new();
    let id = coordinator.begin(recording_request()).unwrap();

    coordinator.cancel(id).unwrap();

    assert!(coordinator.authorize(id).is_err());
    assert_eq!(coordinator.session().unwrap().state(), CaptureSessionState::Cancelled);
}

#[test]
fn session_identifiers_increase_after_terminal_sessions() {
    let mut coordinator = CaptureCoordinator::new();
    let first = coordinator.begin(recording_request()).unwrap();
    coordinator.cancel(first).unwrap();
    let second = coordinator.begin(recording_request()).unwrap();

    assert!(second.value() > first.value());
}
