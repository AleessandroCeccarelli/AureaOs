// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Shared, internally consistent time context for desktop consumers.

use super::{TimeSyncSnapshot, TimeZoneSnapshot, UnixTime};

/// A coherent view of wall time, the active time zone and synchronization health.
///
/// Consumers such as Lumen, day/night policies, notifications and calendars
/// should use this shared snapshot instead of maintaining independent clocks.
/// The zone offset and DST flag must be resolved for the included instant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeContextSnapshot {
    instant: UnixTime,
    time_zone: TimeZoneSnapshot,
    synchronization: TimeSyncSnapshot,
}

impl TimeContextSnapshot {
    pub const fn new(
        instant: UnixTime,
        time_zone: TimeZoneSnapshot,
        synchronization: TimeSyncSnapshot,
    ) -> Self {
        Self {
            instant,
            time_zone,
            synchronization,
        }
    }

    pub const fn instant(&self) -> UnixTime {
        self.instant
    }

    pub const fn time_zone(&self) -> &TimeZoneSnapshot {
        &self.time_zone
    }

    pub const fn synchronization(&self) -> TimeSyncSnapshot {
        self.synchronization
    }
}

/// Read-only source of the shared desktop time context.
pub trait TimeContextProvider: Send + Sync {
    /// Returns a coherent snapshot whose zone rules match its wall-clock instant.
    fn current_snapshot(&self) -> Result<TimeContextSnapshot, TimeContextError>;
}

/// Failure to obtain a usable system time context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeContextError {
    Unavailable,
    InconsistentSnapshot,
}

#[cfg(test)]
mod tests {
    use super::{TimeContextError, TimeContextSnapshot};
    use crate::time::{
        TimeSyncSnapshot, TimeSyncStatus, TimeSyncSource, TimeZoneId, TimeZoneSnapshot,
        UnixTime, UtcOffset,
    };

    #[test]
    fn context_keeps_wall_time_zone_and_sync_state_together() {
        let instant = UnixTime::new(1_800_000_000, 0).unwrap();
        let zone = TimeZoneSnapshot::new(
            TimeZoneId::new("Europe/Rome").unwrap(),
            UtcOffset::from_seconds(3_600).unwrap(),
            Some(false),
        );
        let sync = TimeSyncSnapshot::new(
            TimeSyncStatus::Synchronized,
            TimeSyncSource::NetworkTimeProtocol,
            Some(1_799_999_000),
        );

        let context = TimeContextSnapshot::new(instant, zone, sync);
        assert_eq!(context.instant(), instant);
        assert_eq!(context.time_zone().zone().as_str(), "Europe/Rome");
        assert_eq!(context.synchronization().status(), TimeSyncStatus::Synchronized);
    }

    #[test]
    fn provider_errors_are_explicit() {
        assert_ne!(
            TimeContextError::Unavailable,
            TimeContextError::InconsistentSnapshot
        );
    }
}
