<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# System Time and Time Zones — Exploratory Architecture

## Project status

AureaOS is in an early development phase. This document records the current system-time direction and API contract, not a completed daemon or a frozen implementation specification.

## Goals

The desktop must display reliable local time and date, support user-selected time zones, handle daylight-saving transitions correctly, expose synchronization health, and remain usable offline.

Time is a system service concern, not a widget-only feature. Other components may depend on it for schedules, logs, certificates, updates, backups and environmental policies such as Aurea Lumen.

## Three distinct concepts

### 1. Wall clock

The system wall clock represents an absolute instant, exposed as Unix time (UTC). Changing the selected time zone must never change that instant.

Manual clock setting is a separate privileged system operation. The desktop API describes state and preferences; it must not allow an ordinary application to set the host clock directly.

### 2. Monotonic clock

Elapsed-time measurement must use a monotonic clock, not wall time. Animations, timeouts, retry delays and performance measurements must not jump when the wall clock is corrected or the time zone changes.

The compositor may continue using Smithay's monotonic clock for protocol timestamps. It must not use local civil time for input serials or frame timing.

### 3. Civil time and time zones

The selected zone is an IANA time-zone identifier, such as `Europe/Rome`. The active time-zone database provider resolves the offset and daylight-saving status for a particular instant.

A UTC offset is not a time zone. It is only the offset that applies at a specific instant. The same zone can have different offsets across the year.

AureaOS must not implement daylight-saving rules by hard-coded country tables or fixed UTC offsets. Rules change and may be updated by governments; use the installed, updateable IANA tz database (tzdata) through a maintained system provider. The API's identifier validation checks shape only; the provider must confirm that a zone exists.

## Synchronization and offline behavior

The system should expose whether time is unknown, unsynchronized, synchronizing, synchronized or unavailable, along with the reported source and last successful synchronization when known.

The initial Debian integration should reuse a supported system time service rather than run a second competing NTP daemon. Candidate backends include systemd-timesyncd or chrony; the selected service must be explicit in the base package profile and its status surfaced through a narrow Aurea adapter.

The desktop must not claim that time is synchronized merely because networking is connected. If no network source is available, the system continues using its current clock and clearly reports degraded/unsynchronized status. Manual time configuration remains available through an authorized system service.

RTC policy (UTC versus local time), boot-time correction, suspend/resume behavior, large clock corrections and drift handling must be validated on real hardware before release.

## Ownership boundaries — current proposal

- **Desktop API:** portable time, time-zone, synchronization-status and display-preference contracts.
- **System time provider:** reads wall-clock state and synchronization health from the host.
- **Time-zone provider:** resolves IANA zone rules from the system tz database.
- **Privileged system service:** applies time-zone and manual wall-clock changes after authorization.
- **Shell / Settings:** presents time, date, zone selection, 12/24-hour format and synchronization status.
- **Compositor:** uses monotonic time for protocol and rendering behavior; it does not own system civil time.
- **Other services:** consume the shared provider contract instead of implementing their own clock or DST logic.

AureaOS may later build its own coordination service or user experience where existing Linux services leave a real gap. It must not create duplicate NTP daemons or duplicate tz databases without a demonstrated technical reason.

## Desktop behavior

The proposed settings surface should support:

- Automatic time synchronization on/off, subject to system policy.
- Manual date/time setting when authorized.
- Automatic or manually selected IANA time zone.
- Searchable zone selection, with city/region labels.
- 12-hour or 24-hour clock.
- Date format and optional seconds display.
- Clear synchronization status and last successful synchronization.
- An explicit offline/degraded state without hiding the clock.

Automatic zone detection is optional and must not silently assume precise location. The user must be able to select a zone manually and retain that choice offline.

## Current API foundation

`components/aurea-desktop-api/src/time.rs` currently defines:

- Validated IANA-style zone identifier shape.
- Instant-specific zone snapshot with UTC offset and optional DST status.
- Bounded UTC offset type.
- Unix wall-clock timestamp with normalized nanoseconds.
- Synchronization source, health and last-successful-sync snapshot.
- Automatic/manual synchronization preference.
- Clock and date display preferences.

These are data contracts only. They do not yet read the system clock, resolve tzdb rules, synchronize time, set the RTC, persist settings, or expose IPC.

## Validation still required

- Select and package one initial synchronization backend for Debian Trixie.
- Confirm the installed tzdata source and update lifecycle.
- Define the privileged system interface and authorization model.
- Decide automatic time-zone detection and privacy behavior.
- Test DST gaps and repeated local times, leap seconds policy, clock jumps, RTC modes, suspend/resume and offline boot.
- Validate locale-aware date formatting and accessibility.
- Add integration tests against real provider behavior.

The design remains open to change as these experiments provide evidence.
