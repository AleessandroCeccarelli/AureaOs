// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Logical geometry calculations for registered outputs.

use super::{OutputManager, OutputManagerError};
use crate::output::{
    DisplayOutput, OutputId,
    coordinates::{LogicalPoint, LogicalSize, OutputLogicalBounds},
};

impl OutputManager {
    /// Returns the enabled output's half-open bounds in global logical space.
    ///
    /// The dimensions use the configured fractional scale when present, or
    /// the legacy integer scale otherwise. Disabled outputs have no active
    /// bounds; unknown IDs are reported as errors.
    pub fn logical_bounds(
        &self,
        output_id: OutputId,
    ) -> Result<Option<OutputLogicalBounds>, OutputManagerError> {
        let output = self
            .registry
            .get(output_id)
            .ok_or(OutputManagerError::UnknownOutput(output_id))?;

        if !output.is_enabled() {
            return Ok(None);
        }

        let position = self
            .layout
            .get(output_id)
            .ok_or(OutputManagerError::UnknownOutput(output_id))?;
        let mode = output.current_mode();
        let (physical_width, physical_height) = if swaps_dimensions(output.transform()) {
            (mode.height(), mode.width())
        } else {
            (mode.width(), mode.height())
        };
        let size = LogicalSize {
            width: logical_extent(output, physical_width)?,
            height: logical_extent(output, physical_height)?,
        };
        let origin = LogicalPoint {
            x: i64::from(position.x()),
            y: i64::from(position.y()),
        };

        let bounds = OutputLogicalBounds::from_logical_size(origin, size)
            .ok_or(OutputManagerError::PositionOverflow)?;
        let maximum_exclusive = i64::from(i32::MAX) + 1;
        let right = bounds
            .origin()
            .x
            .checked_add(i64::from(bounds.size().width))
            .ok_or(OutputManagerError::PositionOverflow)?;
        let bottom = bounds
            .origin()
            .y
            .checked_add(i64::from(bounds.size().height))
            .ok_or(OutputManagerError::PositionOverflow)?;
        if right > maximum_exclusive || bottom > maximum_exclusive {
            return Err(OutputManagerError::PositionOverflow);
        }

        Ok(Some(bounds))
    }
}

pub(crate) fn logical_width(output: &DisplayOutput) -> Result<i32, OutputManagerError> {
    let mode = output.current_mode();
    let physical_width = if swaps_dimensions(output.transform()) {
        mode.height()
    } else {
        mode.width()
    };
    let logical_width = logical_extent(output, physical_width)?;
    i32::try_from(logical_width).map_err(|_| OutputManagerError::PositionOverflow)
}

fn swaps_dimensions(transform: smithay::utils::Transform) -> bool {
    matches!(
        transform,
        smithay::utils::Transform::_90
            | smithay::utils::Transform::_270
            | smithay::utils::Transform::Flipped90
            | smithay::utils::Transform::Flipped270
    )
}

fn logical_extent(output: &DisplayOutput, physical: u32) -> Result<u32, OutputManagerError> {
    let logical = if let Some(scale) = output.fractional_scale() {
        scale.physical_to_logical(i64::from(physical))
    } else {
        let scale = u64::from(output.scale());
        if scale == 0 {
            return Err(OutputManagerError::PositionOverflow);
        }

        let physical = u64::from(physical);
        i64::try_from(
            physical
                .checked_add(scale / 2)
                .ok_or(OutputManagerError::PositionOverflow)?
                / scale,
        )
        .ok()
    }
    .ok_or(OutputManagerError::PositionOverflow)?;

    u32::try_from(logical)
        .ok()
        .filter(|extent| *extent > 0)
        .ok_or(OutputManagerError::PositionOverflow)
}
