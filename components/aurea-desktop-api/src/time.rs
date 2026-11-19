// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Technology-independent contracts for system time and time-zone presentation.
//!
//! This module deliberately does not implement a clock daemon, NTP client,
//! calendar conversion engine or time-zone database. Those belong to system
//! providers. In particular, local civil time must be derived from an IANA
//! time-zone database rather than hard-coded daylight-saving rules.

/// Maximum UTC offset accepted by the IANA time-zone model: 18 hours.
pub const MAX_UTC_OFFSET_SECONDS: i32 = 18 * 60 * 60;

/// A validated IANA time-zone identifier, for example `Europe/Rome`.
///
/// This type validates identifier shape only. Existence and rule resolution
/// must be confirmed by the active time-zone database provider.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TimeZoneId(String);

/// Errors returned when a time-zone identifier is malformed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeZoneIdError {
    Empty,
    AbsolutePath,
    InvalidSegment,
    UnsupportedCharacter,
}

impl TimeZoneId {
    /// Validates an IANA-style relative identifier without resolving its rules.
    pub fn new(value: impl Into<String>) -> Result<Self, TimeZoneIdError> {
        let value = value.into();
        if value.is_empty() {
            return Err(TimeZoneIdError::Empty);
        }
        if value.starts_with('/') || value.starts_with('\\') {
            return Err(TimeZoneIdError::AbsolutePath);
        }
        if !value.is_ascii() {
            return Err(TimeZoneIdError::UnsupportedCharacter);
        }

        for segment in value.split('/') {
            if segment.is_empty() || segment == "." || segment == ".." {
                return Err(TimeZoneIdError::InvalidSegment);
            }
            if !segment.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'+')
            }) {
                return Err(TimeZoneIdError::UnsupportedCharacter);
            }
        }

        Ok(Self(value))
    }

    /// Returns the canonical identifier string supplied by the caller.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A UTC offset in seconds east of UTC, validated to the civil-time range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UtcOffset {
    seconds: i32,
}

/// Errors returned for offsets outside the supported range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UtcOffsetError {
    OutOfRange,
}

impl UtcOffset {
    /// Creates an offset between -18:00 and +18:00 inclusive.
    pub const fn from_seconds(seconds: i32) -> Result<Self, UtcOffsetError> {
        if seconds < -MAX_UTC_OFFSET_SECONDS || seconds > MAX_UTC_OFFSET_SECONDS {
            return Err(UtcOffsetError::OutOfRange);
        }
        Ok(Self { seconds })
    }

    /// Returns seconds east of UTC.
    pub const fn seconds(self) -> i32 {
        self.seconds
    }
}

/// Provider-resolved information for a zone at one specific instant.
///
/// The provider must resolve this from its current time-zone database. The
/// offset is not a permanent property of the zone and can change with the date.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeZoneSnapshot {
    zone: TimeZoneId,
    offset: UtcOffset,
    daylight_saving: Option<bool>,
}

impl TimeZoneSnapshot {
    pub const fn new(
        zone: TimeZoneId,
        offset: UtcOffset,
        daylight_saving: Option<bool>,
    ) -> Self {
        Self {
            zone,
            offset,
            daylight_saving,
        }
    }

    pub fn zone(&self) -> &TimeZoneId {
        &self.zone
    }

    pub const fn offset(&self) -> UtcOffset {
        self.offset
    }

    /// None means the provider cannot determine DST status for this zone.
    pub const fn daylight_saving(&self) -> Option<bool> {
        self.daylight_saving
    }
}

/// Source currently responsible for synchronizing the system wall clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeSyncSource {
    #[default]
    Unknown,
    NetworkTimeProtocol,
    Manual,
    HardwareClock,
}

/// Synchronization health reported by the system time provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeSyncStatus {
    #[default]
    Unknown,
    Unsynchronized,
    Synchronizing,
    Synchronized,
    Unavailable,
}

/// Read-only status of the system clock synchronization subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TimeSyncSnapshot {
    status: TimeSyncStatus,
    source: TimeSyncSource,
    last_successful_sync_unix_seconds: Option<i64>,
}

impl TimeSyncSnapshot {
    pub const fn new(
        status: TimeSyncStatus,
        source: TimeSyncSource,
        last_successful_sync_unix_seconds: Option<i64>,
    ) -> Self {
        Self {
            status,
            source,
            last_successful_sync_unix_seconds,
        }
    }

    pub const fn status(self) -> TimeSyncStatus {
        self.status
    }

    pub const fn source(self) -> TimeSyncSource {
        self.source
    }

    pub const fn last_successful_sync_unix_seconds(self) -> Option<i64> {
        self.last_successful_sync_unix_seconds
    }
}

/// User-facing clock format preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClockFormat {
    #[default]
    TwentyFourHour,
    TwelveHour,
}

/// User-facing date presentation preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DateFormat {
    YearMonthDay,
    MonthDayYear,
    #[default]
    DayMonthYear,
}

/// Preferences for displaying date and time in the desktop shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClockPreferences {
    clock_format: ClockFormat,
    date_format: DateFormat,
    show_seconds: bool,
}

impl ClockPreferences {
    pub const fn new(
        clock_format: ClockFormat,
        date_format: DateFormat,
        show_seconds: bool,
    ) -> Self {
        Self {
            clock_format,
            date_format,
            show_seconds,
        }
    }

    pub const fn clock_format(self) -> ClockFormat {
        self.clock_format
    }

    pub const fn date_format(self) -> DateFormat {
        self.date_format
    }

    pub const fn show_seconds(self) -> bool {
        self.show_seconds
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ClockFormat, ClockPreferences, DateFormat, TimeSyncSource, TimeSyncStatus,
        TimeZoneId, TimeZoneIdError, TimeZoneSnapshot, UtcOffset, UtcOffsetError,
    };

    #[test]
    fn accepts_common_iana_zone_identifiers() {
        let zone = TimeZoneId::new("Europe/Rome").unwrap();
        assert_eq!(zone.as_str(), "Europe/Rome");
        assert_eq!(TimeZoneId::new("Etc/UTC").unwrap().as_str(), "Etc/UTC");
    }

    #[test]
    fn rejects_unsafe_or_malformed_zone_identifiers() {
        assert_eq!(TimeZoneId::new(""), Err(TimeZoneIdError::Empty));
        assert_eq!(
            TimeZoneId::new("/etc/localtime"),
            Err(TimeZoneIdError::AbsolutePath)
        );
        assert_eq!(
            TimeZoneId::new("Europe/../UTC"),
            Err(TimeZoneIdError::InvalidSegment)
        );
        assert_eq!(
            TimeZoneId::new("Europe//Rome"),
            Err(TimeZoneIdError::InvalidSegment)
        );
        assert_eq!(
            TimeZoneId::new("Europe/Romè"),
            Err(TimeZoneIdError::UnsupportedCharacter)
        );
    }

    #[test]
    fn validates_utc_offset_boundaries() {
        assert_eq!(UtcOffset::from_seconds(0).unwrap().seconds(), 0);
        assert_eq!(UtcOffset::from_seconds(64_800).unwrap().seconds(), 64_800);
        assert_eq!(UtcOffset::from_seconds(-64_800).unwrap().seconds(), -64_800);
        assert_eq!(UtcOffset::from_seconds(64_801), Err(UtcOffsetError::OutOfRange));
        assert_eq!(UtcOffset::from_seconds(-64_801), Err(UtcOffsetError::OutOfRange));
    }

    #[test]
    fn zone_snapshot_keeps_zone_and_instant_specific_offset_together() {
        let zone = TimeZoneId::new("Europe/Rome").unwrap();
        let offset = UtcOffset::from_seconds(7_200).unwrap();
        let snapshot = TimeZoneSnapshot::new(zone, offset, Some(true));

        assert_eq!(snapshot.zone().as_str(), "Europe/Rome");
        assert_eq!(snapshot.offset().seconds(), 7_200);
        assert_eq!(snapshot.daylight_saving(), Some(true));
    }

    #[test]
    fn sync_status_distinguishes_unknown_from_synchronized() {
        let unknown = super::TimeSyncSnapshot::default();
        assert_eq!(unknown.status(), TimeSyncStatus::Unknown);
        assert_eq!(unknown.source(), TimeSyncSource::Unknown);

        let synced = super::TimeSyncSnapshot::new(
            TimeSyncStatus::Synchronized,
            TimeSyncSource::NetworkTimeProtocol,
            Some(1_800_000_000),
        );
        assert_eq!(synced.status(), TimeSyncStatus::Synchronized);
        assert_eq!(synced.source(), TimeSyncSource::NetworkTimeProtocol);
        assert_eq!(synced.last_successful_sync_unix_seconds(), Some(1_800_000_000));
    }

    #[test]
    fn clock_preferences_have_predictable_defaults() {
        let defaults = ClockPreferences::default();
        assert_eq!(defaults.clock_format(), ClockFormat::TwentyFourHour);
        assert_eq!(defaults.date_format(), DateFormat::DayMonthYear);
        assert!(!defaults.show_seconds());

        let custom = ClockPreferences::new(
            ClockFormat::TwelveHour,
            DateFormat::YearMonthDay,
            true,
        );
        assert!(custom.show_seconds());
    }
}
