// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Application-facing calendar contracts.
//!
//! Calendar storage and synchronization belong to system providers. This
//! module defines portable event data and read/query boundaries only.

use crate::time::{TimeZoneId, UnixTime};

/// Opaque identifier assigned by the calendar provider.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CalendarEventId(String);

impl CalendarEventId {
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return None;
        }
        Some(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A timed calendar event represented by absolute instants.
///
/// The time zone is retained for correct local presentation and editing.
/// It must not be replaced with a fixed UTC offset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarEvent {
    id: CalendarEventId,
    title: String,
    starts_at: UnixTime,
    ends_at: UnixTime,
    time_zone: TimeZoneId,
}

impl CalendarEvent {
    pub fn new(
        id: CalendarEventId,
        title: impl Into<String>,
        starts_at: UnixTime,
        ends_at: UnixTime,
        time_zone: TimeZoneId,
    ) -> Option<Self> {
        let title = title.into();
        if title.trim().is_empty()
            || starts_at.seconds() > ends_at.seconds()
            || (starts_at.seconds() == ends_at.seconds()
                && starts_at.nanoseconds() > ends_at.nanoseconds())
        {
            return None;
        }
        Some(Self {
            id,
            title,
            starts_at,
            ends_at,
            time_zone,
        })
    }

    pub fn id(&self) -> &CalendarEventId {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub const fn starts_at(&self) -> UnixTime {
        self.starts_at
    }

    pub const fn ends_at(&self) -> UnixTime {
        self.ends_at
    }

    pub fn time_zone(&self) -> &TimeZoneId {
        &self.time_zone
    }
}

/// Query for events overlapping an absolute time interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarRange {
    starts_at: UnixTime,
    ends_at: UnixTime,
}

impl CalendarRange {
    pub const fn new(starts_at: UnixTime, ends_at: UnixTime) -> Option<Self> {
        if starts_at.seconds() > ends_at.seconds()
            || (starts_at.seconds() == ends_at.seconds()
                && starts_at.nanoseconds() > ends_at.nanoseconds())
        {
            return None;
        }
        Some(Self { starts_at, ends_at })
    }

    pub const fn starts_at(self) -> UnixTime {
        self.starts_at
    }

    pub const fn ends_at(self) -> UnixTime {
        self.ends_at
    }
}

/// Read-only calendar access. The IPC layer must authorize each application
/// before forwarding a query; a provider is not an authorization boundary.
pub trait CalendarProvider: Send + Sync {
    fn events_in_range(&self, range: CalendarRange) -> Result<Vec<CalendarEvent>, CalendarError>;
}

/// Calendar provider failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarError {
    Unavailable,
    InvalidRange,
    AccessDenied,
}
