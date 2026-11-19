<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Development Log

This is the chronological engineering diary for the AureaOS repository. It complements, rather than replaces, the architecture documents, security audits, roadmap, and Git history.

## 2026-11-18 — Repository baseline and display-scale foundation

### Repository state

- The active development branch is `main`.
- The repository currently has no other branches and no open pull requests.
- The Rust workspace contains four crates: `aurea-desktop`, `aurea-shell`, `aurea-desktop-api`, and `aurea-compositor`.
- The repository already has dedicated architecture, product, security, testing, and human-interface documentation. This entry adds a chronological record instead of duplicating those documents.
- No source files or branches were removed as part of this review: the inspection did not establish that any tracked item was obsolete or safe to delete.

### Engineering work recorded

Recent commits establish the initial display-coordinate and fractional-scale foundation:

- Exact rational scale representation and checked coordinate conversion.
- Separation of global logical coordinates from output-local physical pixels.
- Logical output placement based on scale-aware widths.
- Carrying the rational scale into render targets and pointer geometry.
- Registration of the Wayland fractional-scale manager global.
- Conservative initial preferred scale of 1.0 for newly created fractional-scale objects.

Relevant commits include:

- `9796c023` — document display coordinate foundation and migration status.
- `26f47d60` — define the Linux desktop interoperability contract.
- `6bebd8f1` and `d77db70d` — logical output placement and scale-aware placement tests.
- `8a0df018`, `9cce2778`, `2ae83868`, `d9ff4e36`, and `91f2b214` — fractional-scale protocol and manager-state work.
- `12cddaa8` — fix compositor fractional-scale state access and formatting.

### Verification

GitHub Actions run [44290834407](https://github.com/AleessandroCeccarelli/AureaOs/actions/runs/44290834407), for commit `12cddaa8fe35dff6d8b5e1eece5c5f70ce267424`, completed successfully on 2026-11-18.

All three jobs passed:

- Rust formatting.
- Workspace build and tests.
- Rust dependency security audit.

This is CI evidence for the checks configured in that workflow. It is not evidence of a successful boot on physical hardware, complete DRM/KMS scanout, a finished graphical desktop, or a bootable installation ISO.

### Known limitations and open work

- Fractional scaling is not end-to-end. Output-aware preferred-scale updates, surface buffer/viewport handling, cursor hotspots, clipping, hit-testing, and full renderer integration remain to be connected and validated.
- Physical DRM/KMS output and scanout still require testing on supported hardware.
- The Debian bootstrap and rootfs publication flow still require a complete end-to-end validation on the supported Fedora build host.
- The security audit dated 2026-11-13 records open hardening work. Its findings must not be treated as closed merely because the Rust dependency-audit CI job passes.
- The RustSec advisory affecting transitive `cgmath 0.18.0` remains documented in the security audit and needs an upstream-aware resolution; a successful audit job is not the same as a warning-free dependency report.

### Next engineering actions

1. Continue the fractional-scale integration by inspecting surface/output association and the relevant Smithay 0.7 APIs before changing behavior.
2. Define and test a consistent output-enter/leave and preferred-scale policy for toplevels, popups, and child surfaces.
3. Complete the buffer, viewport, cursor, clipping, and hit-testing integration in small, independently testable modules.
4. Preserve the explicit distinction between CI validation and physical-hardware validation in project documentation.
5. Revisit the open security-audit findings and schedule the Fedora rootfs/bootstrap validation when the build machine is available.

---

## Entry template

Use this structure for future entries:

- **Date and scope**
- **Repository/commit references**
- **Changes actually committed**
- **Verification performed and exact result**
- **Known limitations or regressions**
- **Next actions**

Do not record intentions as completed work.


## 2026-11-19 — Shared input and appearance API foundations

### Repository references

- Active branch: `main`.
- Input API grouping and compatibility export: `872229a`, `47dc3ea`.
- Shared pointer settings consumed by the compositor: `0cfeab6`.
- Appearance token roles and provider contract: `8ee9ff3`.
- Observable device, pointer and keyboard contracts: `aa2de73`, `0250c5c`, `b187afa`, `9446aca`.
- Updated keyboard/pointer policy and Human Interfaces status: `6033a4b`, `6aae467`.

### Changes committed

- Grouped technology-independent keyboard, pointer, device and vendor-extension contracts under `aurea_desktop_api::input`; retained the original keyboard import path as a compatibility re-export.
- Added device descriptors, verified capability snapshots and lifecycle notifications without exposing serial numbers or unrestricted raw input streams.
- Added shared pointer settings and semantic button identifiers; the compositor now consumes the API's pointer-settings type instead of maintaining a duplicate definition.
- Added observable contracts for keyboard-layout selection and pointer settings.
- Added an appearance contract for system/light/dark preference, resolved color scheme, accessibility appearance flags and semantic color roles mapped to the canonical Human Interfaces token names.
- Extended the vendor extension descriptor to match both vendor and product identifiers.
- Expanded the input policy to cover standard mouse behavior and the separation between normal Wayland application input and privileged global input services.

### Verification and limitations

- Repository files and commit contents were inspected after publication.
- No local Cargo build or test result is claimed for these commits. GitHub Actions is configured to run on source changes pushed to `main`; the result for this change set has not been independently confirmed in this entry.
- Fedora hardware validation has not been performed.
- These are public Rust contracts only. Device discovery/provider implementations, runtime layout switching, persisted pointer settings, application-facing IPC/portal transport, native Human Interfaces widgets, and the out-of-process vendor extension manager remain future implementation work.
- The appearance API identifies canonical semantic tokens; actual palette values remain owned by `aurea-human-interfaces/design-system/tokens/index.css`.


### Follow-up — API contract review

- Added observable device lifecycle, keyboard-layout selection and pointer-preference contracts, plus verified vertical-scroll capability.
- Added a focused test for capability deduplication and clarified that additional pointer buttons use normalized identifiers rather than raw kernel event codes.
- Updated the desktop interoperability contract to distinguish internal Rust contracts, the future native Human Interfaces library and the still-unimplemented application-facing IPC/portal layer.
- Updated the roadmap to reflect the new input and appearance API foundations.
- Latest API-focused code commits: `4022516` and `9d2bc14`. GitHub Actions is configured to trigger for source changes pushed to `main`; the run result is not yet confirmed here.


## 2026-11-19 — Shared display API foundation

### Scope and repository alignment

The existing architecture decisions were reviewed before implementation:

- [Multi-Monitor Policy](../architecture/MULTI-MONITOR-POLICY.md) remains authoritative: a maximum of four active outputs, while additional connected displays may remain discovered but disabled.
- [Display Intelligence](../architecture/DISPLAY-INTELLIGENCE.md) remains authoritative for automatic/manual management, fixed refresh, VRR capability checks, safe fallback, and the boundary between policy and compositor/DRM ownership.
- Existing compositor output identity, mode discovery, registration, logical layout, rational scaling, and four-output enforcement remain the implementation foundations. No duplicate compositor output manager or second DRM owner was introduced.

### API files added

Under `components/aurea-desktop-api/src/display/`:

- `modes.rs`: validated pixel dimensions and millihertz refresh values, preserving fractional rates without floating-point identity comparisons.
- `capabilities.rs`: stable display IDs, connector type, verified VRR range, HDR/bit-depth and physical-size capability fields, and public display descriptors.
- `layout.rs`: logical positions, rational per-display scale, enablement and primary-display layout.
- `policy.rs`: Automatic/Manual management, Automatic/Fixed/PreferVariable refresh preference, active refresh strategy and compositor policy context.
- `events.rs`: connected/disconnected/capability/configuration notifications.
- `manager.rs`: snapshot and transactional configuration contracts, validation of unique displays/layout placements, primary-display validity, and the four-active-display ceiling.

The existing `display.rs` remains the public entry point and retains the shared Night Light color controller. It now exports the display-management contracts without changing existing call sites.

### Safety and correctness rules

- Refresh rates are represented in millihertz; no universal 60/120/144 Hz assumptions are embedded.
- VRR ranges are optional verified capability data. Unknown means unavailable for policy decisions, not guessed from EDID or marketing labels.
- Four active displays are allowed; a fifth connected display can remain represented but disabled.
- Layout validation rejects duplicate IDs, missing/extra placements, and a primary display that is not enabled.
- Display configuration is specified as a single transaction. Implementations must restore the last known working configuration if applying a request fails and report rollback failure separately.
- Automatic policy receives explicit compositor/system context; it must not inspect application pixels or private content.
- These are API contracts only. The concrete provider, DRM/KMS mode-application path, VRR execution, persistence, Settings UI and application-facing IPC remain future work.

### Verification

- Source changes were committed directly to `main`.
- Focused Rust unit tests were added for mode validation, fractional refresh representation, VRR range validation, common scale values, automatic defaults, four-display acceptance, fifth-active-display rejection, duplicate IDs/layouts, and invalid primary-display selection.
- No local Cargo test/build is claimed. The Fedora hardware environment has not been used for validation in this change.


### Display API refinement

- Centralized the four-active-display product limit in `aurea-desktop-api::display::MAX_ACTIVE_DISPLAYS`. The compositor's existing `MAX_ACTIVE_OUTPUTS` symbol is now a compatibility re-export, so policy and enforcement cannot drift.
- Kept layout state (enabled/position/scale) in one `DisplayLayout` source rather than duplicating it in each mode request.
- Made the requested mode optional in Automatic mode so the policy engine can choose a discovered mode. Manual mode and Fixed refresh requests require an explicit mode for each enabled display.
- Represented unknown HDR capability explicitly instead of treating missing information as unsupported.
- Added validation that a non-empty active layout has exactly one enabled primary display, and that output IDs and layout placements are unique and complete.


## 2026-11-19 — Aurea Appearance, Aurea Palette, Aurea Form and Aurea Lumen

### Naming and ownership

- **Aurea Appearance** is the system-wide theme/accessibility appearance contract.
- **Aurea Palette** names the canonical semantic color system.
- **Aurea Form** names the canonical geometry and surface language.
- **Aurea Lumen** names environmental display color adaptation, distinct from Light/Dark theme selection.
- AureaOS Human Interfaces remains the single design system and owner of visual values. No parallel palette or token source was created.

### API changes

- Extended `appearance.rs` with the canonical palette identity, form profile and semantic geometry roles. The appearance snapshot now carries palette/form identity alongside the resolved color scheme and accessibility preferences.
- Added `appearance/lumen.rs` with Manual, Schedule, Solar, Ambient and Automatic modes; validated local-clock intervals; ambient-light thresholds with hysteresis; and clamped warmth preferences.
- Automatic Lumen policy prefers reliable ambient sensing and falls back to an available solar or local schedule. Sensor/time providers and policy execution remain separate implementation responsibilities.
- Existing `NightLightState` and the compositor color adjustment path remain in place as migration foundations; no unverified sensor or solar service is claimed.

### Documentation changes

- Added `docs/architecture/AUREA-APPEARANCE-AND-LUMEN.md`.
- Named Aurea Palette and Aurea Form in Human Interfaces documentation and the color-system specification.
- Reaffirmed `aurea-human-interfaces/design-system/tokens/index.css` as the sole source of visual values.

### Verification status

- Changes committed directly to `main`.
- No local Cargo build/test or completed CI result is claimed in this entry.
- Concrete Appearance provider, ambient sensor discovery, solar/time provider, Lumen coordinator, persistence, native Settings UI and application-facing IPC remain future implementation work.


## 2026-11-19 — Clarify early-stage and provisional design status

AureaOS is explicitly an embryonic project. The recent names and contracts for Aurea Appearance, Aurea Lumen, Aurea Palette and Aurea Form describe the current direction, not a final or immutable product specification.

Updated the architecture document, Human Interfaces overview, design-system README, color-system specification and roadmap to state that:

- Product names, palette values, shapes, visual treatments, functions, APIs, architecture and technology choices may change during development.
- Current visual tokens and API identities are provisional and must not be presented as frozen standards.
- Documentation and API contracts are not evidence of completed runtime services.
- Design proposals, experimental code, implemented API contracts, hardware-validated behavior and release-ready capabilities must be reported separately.
- A compatibility/versioning policy must be established before promising long-term stability to third-party developers.

No runtime code or visual token values were changed in this documentation clarification.


## 2026-11-19 — Lumen preference contract hardening and shell adapter

### Changes committed

- Made `LumenPreferences` fields private so callers cannot bypass the warmth range or replace validated schedule/ambient-threshold values through direct field mutation.
- Added focused getters and setters for enabled state, mode, warmth, schedule and ambient thresholds.
- Added a unit test covering preference updates through the public API.
- Replaced the shell's duplicate Night Light schedule/preference storage with a compatibility adapter over the shared `aurea-desktop-api::appearance::lumen` types.
- Preserved the existing shell-facing Night Light mode names and runtime call pattern during migration. The adapter maps Manual, CustomSchedule and SunsetToSunrise to the corresponding shared Lumen modes.
- The existing shell scheduler remains the source of the current schedule decision. No ambient sensor, solar provider, policy coordinator or new runtime behavior is claimed.

### Verification status

- Changes were committed directly to `main`.
- No local Cargo formatting, build or test result is claimed. The latest GitHub combined-status query did not return individual status checks for the API change.
- The next integration step is to replace the temporary compatibility mode/scheduler interface with an explicit Lumen policy decision once its provider contract is designed and tested.


## 2026-11-19 — Initial system time and time-zone API foundation

### Direction

Defined system time as a shared operating-system concern rather than a shell-only clock widget. Wall-clock time (UTC), monotonic time and local civil-time presentation are explicitly separate concepts. A time-zone identifier is not interchangeable with a fixed UTC offset.

### API changes

- Added `components/aurea-desktop-api/src/time.rs` and exported it from the crate.
- Added an IANA-style `TimeZoneId` shape validator; actual zone existence and rules remain the responsibility of the installed tz database provider.
- Added bounded `UtcOffset` and instant-specific `TimeZoneSnapshot` contracts with optional daylight-saving status.
- Added normalized Unix timestamp representation, synchronization policy/status/source snapshots, and clock/date display preferences.
- Added unit tests for identifier validation, UTC offset boundaries, zone snapshots, normalized timestamps, synchronization state and display defaults.

### Architecture and roadmap

- Added `docs/architecture/SYSTEM-TIME-AND-TIMEZONES.md`.
- Added roadmap milestones for Debian synchronization backend selection, provider integration, privileged configuration, tzdata, shell settings and hardware validation.
- The architecture proposes reusing one supported Debian time synchronization service (systemd-timesyncd or chrony after evaluation) rather than starting a competing NTP daemon.
- The IANA tz database must remain updateable; daylight-saving rules must not be hard-coded.
- Manual zone selection and offline clock operation remain required. Automatic zone detection must respect privacy and must not silently assume precise location.

### Verification status

These are experimental API contracts and documentation only. No daemon, NTP synchronization, tzdb resolver, privileged setter, persistence, shell clock widget or IPC implementation has been added. Cargo formatting, build and tests have not been run in this environment.


### Debian backend decision update

After reviewing Debian 13 documentation, selected `systemd-timesyncd` as the initial network time synchronization backend and added it to `debian/packages/base.list`. Debian documents it as the default client-side synchronization service; chrony remains a possible future alternative only if AureaOS develops requirements that justify it. The architecture specifies using `systemd-timedated` over D-Bus for privileged time/time-zone/RTC configuration rather than launching privileged shell commands from the UI. The installed `tzdata` package remains the source of updateable time-zone rules.

This package-manifest change has not yet been validated by a fresh Debian bootstrap.
