// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Capture Core
// Copyright (C) 2026 Alessandro Ceccarelli

//! Single-session coordinator and lifecycle transition authority.

use crate::{
    CaptureError, CaptureKind, CaptureRequest, CaptureSession, CaptureSessionId,
    CaptureSessionState,
};

/// Coordinates one interactive capture workflow at a time.
///
/// A single active session keeps source selection, permission prompts and
/// recording indicators unambiguous. Completed sessions may be replaced.
#[derive(Debug, Default)]
pub struct CaptureCoordinator {
    next_id: u64,
    session: Option<CaptureSession>,
}

impl CaptureCoordinator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a session that is waiting for explicit user authorization.
    pub fn begin(&mut self, request: CaptureRequest) -> Result<CaptureSessionId, CaptureError> {
        if self.session.is_some_and(|session| !session.state().is_terminal()) {
            return Err(CaptureError::SessionBusy);
        }

        let next_id = self.next_id.checked_add(1).ok_or(CaptureError::SessionIdExhausted)?;
        self.next_id = next_id;

        let id = CaptureSessionId::from_value(next_id);
        self.session = Some(CaptureSession::new(id, request));

        Ok(id)
    }

    pub fn session(&self) -> Option<CaptureSession> {
        self.session
    }

    pub fn authorize(&mut self, id: CaptureSessionId) -> Result<(), CaptureError> {
        self.transition(id, CaptureSessionState::AwaitingAuthorization, CaptureSessionState::Ready)
    }

    pub fn start(&mut self, id: CaptureSessionId) -> Result<(), CaptureError> {
        self.transition(id, CaptureSessionState::Ready, CaptureSessionState::Capturing)
    }

    pub fn pause(&mut self, id: CaptureSessionId) -> Result<(), CaptureError> {
        let session = self.session_for(id)?;
        if !matches!(session.kind(), CaptureKind::Recording { .. }) {
            return Err(CaptureError::InvalidTransition);
        }

        self.transition(id, CaptureSessionState::Capturing, CaptureSessionState::Paused)
    }

    pub fn resume(&mut self, id: CaptureSessionId) -> Result<(), CaptureError> {
        self.transition(id, CaptureSessionState::Paused, CaptureSessionState::Capturing)
    }

    pub fn begin_finalization(&mut self, id: CaptureSessionId) -> Result<(), CaptureError> {
        let session = self.session_for(id)?;
        if !matches!(session.state(), CaptureSessionState::Capturing | CaptureSessionState::Paused) {
            return Err(CaptureError::InvalidTransition);
        }

        self.session_mut_for(id)?.set_state(CaptureSessionState::Finalizing);
        Ok(())
    }

    pub fn complete(&mut self, id: CaptureSessionId) -> Result<(), CaptureError> {
        self.transition(id, CaptureSessionState::Finalizing, CaptureSessionState::Completed)
    }

    pub fn cancel(&mut self, id: CaptureSessionId) -> Result<(), CaptureError> {
        let session = self.session_for(id)?;
        if matches!(session.state(), CaptureSessionState::Finalizing) || session.state().is_terminal() {
            return Err(CaptureError::InvalidTransition);
        }

        self.session_mut_for(id)?.set_state(CaptureSessionState::Cancelled);
        Ok(())
    }

    pub fn fail(&mut self, id: CaptureSessionId) -> Result<(), CaptureError> {
        let session = self.session_for(id)?;
        if session.state().is_terminal() {
            return Err(CaptureError::InvalidTransition);
        }

        self.session_mut_for(id)?.set_state(CaptureSessionState::Failed);
        Ok(())
    }

    fn transition(
        &mut self,
        id: CaptureSessionId,
        expected: CaptureSessionState,
        next: CaptureSessionState,
    ) -> Result<(), CaptureError> {
        let session = self.session_for(id)?;
        if session.state() != expected {
            return Err(CaptureError::InvalidTransition);
        }

        self.session_mut_for(id)?.set_state(next);
        Ok(())
    }

    fn session_for(&self, id: CaptureSessionId) -> Result<&CaptureSession, CaptureError> {
        self.session
            .as_ref()
            .filter(|session| session.id() == id)
            .ok_or(CaptureError::NoActiveSession)
    }

    fn session_mut_for(&mut self, id: CaptureSessionId) -> Result<&mut CaptureSession, CaptureError> {
        self.session
            .as_mut()
            .filter(|session| session.id() == id)
            .ok_or(CaptureError::NoActiveSession)
    }
}

