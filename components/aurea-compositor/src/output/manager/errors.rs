// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Output manager errors and reconciliation results.

use std::fmt;

use crate::output::{OutputId, OutputMode};

/// Errors returned by output management operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputManagerError {
    DuplicateOutput(OutputId),
    UnknownOutput(OutputId),
    InvalidMode { output: OutputId, mode: OutputMode },
    InvalidScale { output: OutputId, scale: u32 },
    TooManyActiveOutputs { maximum: usize },
    PositionOverflow,
}

impl fmt::Display for OutputManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateOutput(output) => write!(f, "Output {output} is already registered"),
            Self::UnknownOutput(output) => write!(f, "Unknown output: {output}"),
            Self::InvalidMode { output, mode } => {
                write!(f, "Mode {mode:?} is not supported by output {output}")
            }
            Self::InvalidScale { output, scale } => {
                write!(f, "Invalid scale {scale} for output {output}")
            }
            Self::TooManyActiveOutputs { maximum } => {
                write!(f, "AureaOS supports at most {maximum} active display outputs")
            }
            Self::PositionOverflow => f.write_str("Output layout position overflow"),
        }
    }
}

impl std::error::Error for OutputManagerError {}

/// IDs affected by a per-device output reconciliation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OutputReconciliation {
    pub added: Vec<OutputId>,
    pub updated: Vec<OutputId>,
    pub removed: Vec<OutputId>,
}
