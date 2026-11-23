// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Reconciles DRM discovery results with the compositor output registry.

use std::collections::{BTreeSet, HashMap, HashSet};

use super::geometry::logical_width;
use super::{OutputManager, OutputManagerError, OutputReconciliation};
use crate::output::{DisplayOutput, OutputId, limits::MAX_ACTIVE_OUTPUTS};

impl OutputManager {
    /// Reconciles outputs owned by one DRM device with a fresh discovery result.
    ///
    /// Unchanged outputs keep their object and layout position. Updated outputs
    /// retain scale, enabled state, position and a still-supported current mode.
    /// New outputs are placed after the rightmost output in the final set.
    pub fn reconcile_outputs(
        &mut self,
        owned_ids: &[OutputId],
        discovered: Vec<DisplayOutput>,
    ) -> Result<OutputReconciliation, OutputManagerError> {
        let owned = owned_ids.iter().copied().collect::<HashSet<_>>();
        let mut desired = BTreeSet::new();

        for output in &discovered {
            let id = output.id();
            if !desired.insert(id) {
                return Err(OutputManagerError::DuplicateOutput(id));
            }
            if self.registry.get(id).is_some() && !owned.contains(&id) {
                return Err(OutputManagerError::DuplicateOutput(id));
            }
        }

        let discovered_ids = discovered
            .iter()
            .map(DisplayOutput::id)
            .collect::<HashSet<_>>();
        let mut changes = OutputReconciliation::default();
        let mut updates = Vec::new();
        let mut additions = Vec::new();
        let mut effective_widths = HashMap::new();

        for output in discovered {
            let id = output.id();
            if let Some(current) = self.registry.get(id) {
                let mut replacement = current.clone();
                replacement.update_from_discovery(output);
                effective_widths.insert(id, logical_width(&replacement)?);
                if *current != replacement {
                    changes.updated.push(id);
                    updates.push((id, replacement));
                }
            } else {
                changes.added.push(id);
                effective_widths.insert(id, logical_width(&output)?);
                additions.push(output);
            }
        }

        changes.removed = owned
            .into_iter()
            .filter(|id| !discovered_ids.contains(id) && self.registry.get(*id).is_some())
            .collect();

        let removed = changes.removed.iter().copied().collect::<HashSet<_>>();
        let retained_active = self
            .registry
            .iter()
            .filter(|output| output.is_enabled() && !removed.contains(&output.id()))
            .count();
        let added_active = additions
            .iter()
            .filter(|output| output.is_enabled())
            .count();
        if retained_active.saturating_add(added_active) > MAX_ACTIVE_OUTPUTS {
            return Err(OutputManagerError::TooManyActiveOutputs {
                maximum: MAX_ACTIVE_OUTPUTS,
            });
        }

        let mut next_x = self.registry.iter().try_fold(0_i32, |right, output| {
            if removed.contains(&output.id()) {
                return Ok(right);
            }
            let position = self.layout.get(output.id()).unwrap_or_default();
            let width = effective_widths
                .get(&output.id())
                .copied()
                .map(Ok)
                .unwrap_or_else(|| logical_width(output))?;
            let edge = position
                .x()
                .checked_add(width)
                .ok_or(OutputManagerError::PositionOverflow)?;
            Ok::<i32, OutputManagerError>(right.max(edge))
        })?;

        let mut addition_positions = Vec::with_capacity(additions.len());
        for output in &additions {
            let width = logical_width(output)?;
            addition_positions.push((output.id(), (next_x, 0)));
            next_x = next_x
                .checked_add(width)
                .ok_or(OutputManagerError::PositionOverflow)?;
        }

        // All validation and position arithmetic is complete before mutation.
        let before = self.clone();
        for id in &changes.removed {
            self.unregister(*id);
        }
        for (id, output) in updates {
            if let Some(current) = self.registry.get_mut(id) {
                *current = output;
            }
        }
        for (output, (_, position)) in additions.into_iter().zip(addition_positions) {
            if let Err(error) = self.register(output, position) {
                *self = before;
                return Err(error);
            }
        }

        Ok(changes)
    }
}
