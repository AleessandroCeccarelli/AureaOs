// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Tracks KMS ownership across atomic validation, commit and page-flip events.

use std::{error::Error, fmt};

/// Stable compositor-side identity for a registered DRM framebuffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DrmFramebufferId(u64);

impl DrmFramebufferId {
    /// Creates an identity from the compositor's monotonically assigned ID.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the underlying identity value.
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Invalid transitions in the KMS presentation lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrmPresentationError {
    TransactionInProgress,
    NoTransaction,
    TestNotPassed,
    CandidateIsCurrent,
    UnexpectedFramebuffer,
}

impl fmt::Display for DrmPresentationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::TransactionInProgress => "A KMS presentation transaction is already in progress",
            Self::NoTransaction => "There is no KMS presentation transaction in progress",
            Self::TestNotPassed => "The atomic TEST_ONLY validation has not succeeded",
            Self::CandidateIsCurrent => "The candidate framebuffer is already active on this CRTC",
            Self::UnexpectedFramebuffer => "The page-flip event does not match the pending framebuffer",
        };
        formatter.write_str(message)
    }
}

impl Error for DrmPresentationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransactionState {
    Testing {
        candidate: DrmFramebufferId,
    },
    Ready {
        candidate: DrmFramebufferId,
    },
    PendingFlip {
        candidate: DrmFramebufferId,
    },
}

/// Tracks the framebuffer currently scanned out and the next framebuffer.
///
/// The tracker deliberately does not own framebuffer objects. The caller
/// retains those objects in its resource registry until this state machine
/// reports that KMS has released them. GPU completion remains a separate
/// condition and must also be satisfied before a resource can be reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DrmPresentationLifecycle {
    current: Option<DrmFramebufferId>,
    transaction: Option<TransactionState>,
}

impl DrmPresentationLifecycle {
    /// Creates an empty presentation lifecycle for one CRTC.
    pub const fn new() -> Self {
        Self {
            current: None,
            transaction: None,
        }
    }

    /// Returns the framebuffer currently considered active on this CRTC.
    pub const fn current(&self) -> Option<DrmFramebufferId> {
        self.current
    }

    /// Returns whether an atomic test/commit transaction is in progress.
    pub const fn has_pending_transaction(&self) -> bool {
        self.transaction.is_some()
    }

    /// Starts validation of a candidate framebuffer before atomic commit.
    pub fn begin_test(
        &mut self,
        candidate: DrmFramebufferId,
    ) -> Result<(), DrmPresentationError> {
        if self.transaction.is_some() {
            return Err(DrmPresentationError::TransactionInProgress);
        }
        if self.current == Some(candidate) {
            return Err(DrmPresentationError::CandidateIsCurrent);
        }

        self.transaction = Some(TransactionState::Testing { candidate });
        Ok(())
    }

    /// Records successful DRM atomic TEST_ONLY validation.
    pub fn test_passed(&mut self) -> Result<(), DrmPresentationError> {
        let Some(TransactionState::Testing { candidate }) = self.transaction else {
            return Err(DrmPresentationError::NoTransaction);
        };

        self.transaction = Some(TransactionState::Ready { candidate });
        Ok(())
    }

    /// Cancels a rejected atomic TEST_ONLY request and returns its candidate.
    pub fn test_failed(&mut self) -> Result<DrmFramebufferId, DrmPresentationError> {
        let Some(TransactionState::Testing { candidate }) = self.transaction else {
            return Err(DrmPresentationError::NoTransaction);
        };

        self.transaction = None;
        Ok(candidate)
    }

    /// Records that the non-blocking KMS commit was accepted by the kernel.
    ///
    /// The candidate must remain alive while the page-flip event is pending.
    pub fn commit_accepted(&mut self) -> Result<(), DrmPresentationError> {
        let Some(TransactionState::Ready { candidate }) = self.transaction else {
            return Err(if self.transaction.is_some() {
                DrmPresentationError::TestNotPassed
            } else {
                DrmPresentationError::NoTransaction
            });
        };

        self.transaction = Some(TransactionState::PendingFlip { candidate });
        Ok(())
    }

    /// Records a rejected KMS commit and returns the candidate for retirement.
    ///
    /// The previously active framebuffer remains owned by KMS.
    pub fn commit_failed(&mut self) -> Result<DrmFramebufferId, DrmPresentationError> {
        let Some(TransactionState::Ready { candidate }) = self.transaction else {
            return Err(if self.transaction.is_some() {
                DrmPresentationError::TestNotPassed
            } else {
                DrmPresentationError::NoTransaction
            });
        };

        self.transaction = None;
        Ok(candidate)
    }

    /// Completes the accepted page flip and returns the previously active FB.
    ///
    /// The returned framebuffer is no longer referenced by this CRTC and may
    /// be retired once its independent GPU ownership has also been resolved.
    pub fn page_flip_complete(
        &mut self,
        displayed: DrmFramebufferId,
    ) -> Result<Option<DrmFramebufferId>, DrmPresentationError> {
        let Some(TransactionState::PendingFlip { candidate }) = self.transaction else {
            return Err(DrmPresentationError::NoTransaction);
        };
        if candidate != displayed {
            return Err(DrmPresentationError::UnexpectedFramebuffer);
        }

        let released = self.current.replace(candidate);
        self.transaction = None;
        Ok(released)
    }
}
