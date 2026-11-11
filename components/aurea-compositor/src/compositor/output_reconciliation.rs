// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Transactional publication of output-manager reconciliations.

use std::error::Error;

use smithay::reexports::wayland_server::DisplayHandle;

use crate::output::{
    OutputId,
    manager::{OutputManager, OutputReconciliation},
    wayland::OutputPublicationError,
};

use super::AureaCompositor;

/// Failure while publishing a committed output reconciliation.
#[derive(Debug)]
pub struct OutputReconciliationError {
    output_id: OutputId,
    source: OutputPublicationError,
    rollback_error: Option<OutputPublicationError>,
}

impl std::fmt::Display for OutputReconciliationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "Failed to publish reconciled output {}: {}",
            self.output_id, self.source
        )?;

        if let Some(error) = &self.rollback_error {
            write!(
                formatter,
                "; restoring prior output globals also failed: {error}"
            )?;
        }

        Ok(())
    }
}

impl Error for OutputReconciliationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

impl AureaCompositor {
    /// Publishes a per-device model reconciliation, rolling back the output
    /// manager and affected Wayland globals if any changed global is invalid.
    pub fn publish_output_reconciliation(
        &mut self,
        display: &DisplayHandle,
        previous: OutputManager,
        changes: &OutputReconciliation,
    ) -> Result<(), OutputReconciliationError> {
        let mut affected = changes.added.clone();
        affected.extend(changes.updated.iter().copied());
        affected.extend(changes.removed.iter().copied());

        for id in &changes.removed {
            self.unpublish_wayland_output(*id);
        }

        for id in changes.added.iter().chain(&changes.updated) {
            if let Err(source) = self.refresh_wayland_output(display, *id) {
                self.output_manager = previous.clone();

                let mut rollback_error = None;

                for rollback_id in &affected {
                    if previous.registry().get(*rollback_id).is_some() {
                        if let Err(error) = self.refresh_wayland_output(display, *rollback_id) {
                            rollback_error.get_or_insert(error);
                        }
                    } else {
                        self.unpublish_wayland_output(*rollback_id);
                    }
                }

                return Err(OutputReconciliationError {
                    output_id: *id,
                    source,
                    rollback_error,
                });
            }
        }

        // Remove frame scheduling state only after reconciliation succeeds.
        for output_id in &changes.removed {
            self.frame_scheduler.remove_output(*output_id);
        }

        Ok(())
    }
}
