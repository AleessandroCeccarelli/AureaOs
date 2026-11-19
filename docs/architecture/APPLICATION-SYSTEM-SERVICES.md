<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Application System Services — Exploratory Architecture

## Purpose

AureaOS exposes shared operating-system capabilities to compatible applications through the Aurea Desktop API and a permission-controlled local IPC boundary. Applications should reuse system services instead of maintaining competing copies of shared state.

This is an early architecture direction. The contracts may evolve before they are considered stable.

## Shared service ownership

| Capability | System-owned responsibility | Application access |
|---|---|---|
| System clock | Wall clock, monotonic clock distinction, synchronization health | Read current time context; no direct clock setting |
| Time zones | IANA zone database, current offset and DST resolution | Read available zones and resolve local presentation |
| Calendar | Local event store, date/time interpretation, reminders | Query authorized calendars and events; create or modify only with grants |
| Notes | Local note store, indexing and persistence | Access selected notes or explicitly granted library scopes |
| Clock preferences | 12/24-hour format, date format, seconds preference | Read shared presentation preferences where appropriate |
| Environmental policies | Day/night scheduling and Lumen coordination | Consume shared time context; no private clock implementation |

The system clock remains the source of absolute time. A selected time zone changes civil-time presentation, not the underlying instant. Monotonic time remains separate and is used for elapsed durations.

## API boundaries

The first technology-independent contracts live in components/aurea-desktop-api:

- time: wall-clock values, zone identifiers, synchronization status and display preferences.
- time/context: a coherent time snapshot and read-only provider contract.
- calendar: event identifiers, timed event values, range queries and provider contract.
- notes: opaque note identifiers, metadata, note operations and requested access levels.

These traits are provider contracts, not running services. They do not yet define a daemon, storage engine, IPC protocol or complete permission broker.

## Application access and privacy

Applications must never receive unrestricted access merely because they link against the API crate.

The future IPC/service boundary must:
- Identify the calling application using a system-controlled identity, not a caller-supplied name.
- Check permissions on every protected operation.
- Support least-privilege scopes and user-mediated selection.
- Distinguish read, create, modify and delete operations.
- Bind selected-item grants to the selected note or calendar resource.
- Allow the user to review and revoke grants.
- Keep private note bodies and calendar details out of broad metadata or discovery APIs.
- Return explicit access-denied and service-unavailable errors.

A calendar widget may receive permission to read event summaries without receiving permission to edit events. A note-taking integration may receive one selected note without being able to enumerate the user's entire notes library.

The Rust API traits do not themselves provide security. Enforcement belongs in the trusted service/IPC layer.

## Data and synchronization policy

The initial design is local-first. Clock, calendar and notes functions must remain usable without a cloud account or network connection. Optional synchronization with external accounts can be added later through separate adapters, conflict handling and explicit user consent.

The system owns persistence and shared policy; applications own their presentation and workflows. Apps must not write directly into private service databases.

## Time-aware behavior

Calendar events are stored against absolute instants for timed occurrences and retain the relevant IANA zone for correct display and editing. Recurring events, all-day events, reminders, locale formatting and daylight-saving ambiguity require dedicated models before those features are implemented.

Lumen, Day/Night, notifications, calendar views and reminders consume the shared time context. They must not each implement their own time-zone or daylight-saving rules.

## Implementation sequence

1. Complete and validate the shared time provider and time-zone resolver.
2. Define the trusted local IPC identity and permission broker.
3. Implement calendar storage/query and reminder scheduling.
4. Implement notes storage and user-mediated sharing.
5. Add application SDK bindings and examples.
6. Add optional external account synchronization only after local behavior and conflict rules are reliable.

No application-facing IPC or calendar/notes storage implementation is claimed by the current API contracts.
