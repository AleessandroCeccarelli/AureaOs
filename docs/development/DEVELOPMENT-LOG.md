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
