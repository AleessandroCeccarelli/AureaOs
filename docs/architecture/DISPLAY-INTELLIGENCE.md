<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Aurea Display Intelligence

## Purpose

Aurea Display Intelligence is the planned AureaOS subsystem for discovering display capabilities and selecting appropriate display modes and refresh strategies.

Its goal is to make display configuration automatic by default while keeping clear, predictable manual controls available to the user.

The Aurea compositor already provides initial DRM device discovery, connector discovery, output lifecycle management, and Wayland output publication.

The technology-independent display API foundation now exists in `components/aurea-desktop-api/src/display/`. It describes modes, verified capabilities, VRR ranges, logical placement, per-display scaling, automatic/manual policy, lifecycle events, and a transactional display-manager contract. This is an API contract, not a running service. Complete DRM/KMS mode setting, physical frame presentation, VRR control, and adaptive refresh policy remain future work.

## Product Principles

- AureaOS, not the Debian base distribution, owns display policy and user-facing decisions.
- Linux kernel drivers and DRM/KMS provide the hardware interfaces used to discover capabilities and apply supported configurations.
- Vulkan is the selected and exclusive graphics rendering API for AureaOS.
- The monitor and graphics driver define the actual limits. AureaOS must never request unsupported resolutions, refresh rates, or VRR ranges.
- Automatic management is the default experience; manual configuration remains available.
- Each connected display is discovered and managed independently.
- Capability discovery and policy decisions are separate from the compositor's rendering and protocol responsibilities.
- Existing compositor responsibilities must not be duplicated by a separate display service.

## Subsystem Architecture

### Display Capability Service

Discovers and maintains a capability profile for each connected display, using information exposed by DRM/KMS, connector properties, EDID, and the active graphics driver.

The current compositor provides initial foundations for this work:

- DRM connector discovery.
- Supported display mode discovery.
- Stable output identity generation.
- Output registration and removal.
- Logical output configuration and layout.

These foundations do not yet constitute a complete display capability service.

The planned capability profile should include, when available:

- Stable display and connector identity.
- Supported resolutions and display modes.
- Available fixed refresh rates for each mode.
- Whether the connector and CRTC path supports variable refresh rate.
- The effective VRR range and relevant driver constraints, when they can be established reliably.
- Physical and logical display properties needed by the desktop.

EDID data is useful input, but it is not by itself a guarantee that a feature is usable. Driver-reported properties and the active display pipeline must also be checked.

Unknown or contradictory capabilities must be treated conservatively.

### Aurea VRR

Aurea VRR is the planned adaptive-refresh strategy for displays and display pipelines that support variable refresh rate.

It should coordinate presentation with the actual frame cadence and the effective VRR limits.

It may allow the display to follow low or high frame rates within the supported range, subject to driver behavior, synchronization, and panel limitations.

The service must not assume that every VRR display supports 1 Hz.

Minimum and maximum rates are hardware- and driver-dependent. Low Framerate Compensation (LFC), when available, may help handle frame rates below the native VRR minimum; it must not be assumed without capability evidence.

VRR support will depend on the complete graphics and presentation pipeline, not simply on detecting a compatible monitor.

### Aurea Fixed Refresh

Aurea Fixed Refresh is the planned fallback strategy for displays that do not support VRR, or for situations where VRR is unavailable or unsuitable.

It selects among the fixed modes exposed by the display and graphics driver.

Selection should consider:

- Active workload.
- Presentation cadence.
- User preferences.
- Power consumption.
- Stability and compatibility.

It must not claim to provide continuous variable refresh on fixed-refresh hardware.

### Display Policy Engine

The Display Policy Engine coordinates capability profiles, user preferences, desktop context, and system constraints to choose a display policy.

Context may include:

- Compositor presentation timing.
- Frame cadence.
- Fullscreen state.
- Known application presentation hints.
- Desktop animation activity.
- Power policy.

The initial design should use explicit system and compositor signals rather than inspecting application pixels or private content.

The policy engine should:

- Choose a valid resolution and refresh strategy for each display.
- Prefer VRR when supported and appropriate.
- Otherwise select a suitable fixed refresh mode.
- Avoid unnecessary mode changes and oscillation through hysteresis, minimum dwell times, and safe transitions.
- Respect manual user choices and explicit application or system constraints.
- Handle display hotplug, session changes, suspend/resume, and driver capability changes.
- Provide a safe fallback if capability discovery or mode application fails.

### DRM/KMS Backend

The DRM/KMS backend is the planned boundary for applying display configuration through the kernel DRM/KMS interfaces and active graphics driver.

It must report success, failure, and the effective state back to the subsystem.

The backend is an implementation boundary, not the policy owner.

AureaOS decides what it wants; DRM/KMS and the driver determine whether the requested configuration can be applied.

The current compositor already manages DRM devices and output lifecycle foundations. The future backend must integrate with this existing ownership model rather than opening competing device-management paths.

### Compositor Integration

The Wayland compositor supplies trusted presentation and output lifecycle information and applies display state changes requested by the display subsystem.

The current compositor already provides initial output lifecycle management and publishes registered outputs as Wayland output globals.

It does not yet provide complete physical display mode setting or frame presentation.

The compositor should not contain the entire adaptive policy engine.

The boundary between the display subsystem, compositor, and DRM/KMS backend must be explicit. The design must avoid competing owners of the same DRM device or display state.

## User Experience

The planned **Settings → Display & Video** interface should provide:

- **Automatic — AureaOS:** recommended default. AureaOS chooses a suitable resolution and refresh strategy using detected capabilities and current context.
- **Manual:** lets the user choose a supported resolution and fixed refresh rate where available.
- A clear indication of the active display, supported modes, VRR availability, and current management mode.
- A safe confirmation and revert flow when applying a manual display configuration, so an unusable mode does not leave the user without a working display.
- Per-display settings for multi-monitor systems.

The UI must distinguish a selected fixed mode from a live VRR operating frequency.

With VRR, the selected display mode defines the usable mode and VRR envelope; the instantaneous refresh can vary during presentation.

This interface is a product requirement, not an implemented feature.

## Operating Rules

1. Discover capabilities before offering choices.
2. Treat the effective driver and display pipeline as authoritative for what can actually be applied.
3. Never infer a universal refresh range from marketing labels or EDID alone.
4. Keep resolution selection and refresh policy related but distinct: not every resolution supports every refresh rate or VRR range.
5. Prefer stable transitions over frequent mode switching.
6. Make automatic behavior explainable and allow the user to override it.
7. Fall back to a known working fixed mode if VRR or adaptive policy cannot be used.
8. Re-evaluate capabilities after hotplug, resume, GPU/session changes, and relevant driver events.
9. Never apply a display configuration without a validated path to restore a working state.

## Multi-Monitor Behavior

Each display has its own capability profile and policy state.

AureaOS should be able to use VRR on one display and fixed refresh on another.

Shared GPU, CRTC, bandwidth, synchronization, and compositor constraints must be considered before applying simultaneous configurations.

AureaOS supports up to four active outputs as a deliberate balance between desktop usability and compositor maintainability. The limit applies to enabled outputs; additional connected displays may be discovered but remain disabled. The rationale and operational rules are documented in [Multi-Monitor Policy](MULTI-MONITOR-POLICY.md).

The implementation must also account for the possibility that different outputs have different capabilities and limitations.

## Implementation Direction

The work should be delivered incrementally:

1. **Display identity and output foundations — initial implementation available.** The compositor already provides stable output identities, connector discovery, output registration, removal, and logical layout management.
2. **DRM device and output lifecycle — initial implementation available.** Device discovery, lifecycle integration, output reconciliation, and Wayland output publication are present in the compositor.
3. **Physical display pipeline — incomplete.** Implement the Vulkan rendering and presentation path and a DRM/KMS mode-application boundary with explicit ownership and error reporting.
4. **Capability reporting — API contract defined; provider incomplete.** Extend discovered output information into reliable per-display capability profiles and implement the service provider behind the shared desktop API. A hardware-independent mode selector now chooses only from reported modes, requires an exact supported mode in Manual mode, and uses VRR only when a verified effective range is covered by the selected mode.
5. **Fixed-refresh configuration — policy foundation implemented; KMS application planned.** The shared API now provides deterministic automatic mode selection using supported resolution, measured frame cadence, animation/fullscreen context, and battery state. The selected mode is not yet applied to a physical KMS output.
6. **VRR capability detection — planned.** Detect effective support across the complete display pipeline.
7. **Controlled VRR enablement — planned.** Add VRR only where the complete pipeline supports it and safe transitions can be guaranteed.
8. **Display policy — planned.** Implement context-aware policy, safeguards, and per-display settings.
9. **Validation — required.** Test different drivers, monitors, hotplug, suspend/resume, and multi-monitor configurations.

No specific minimum or maximum refresh rate is guaranteed by AureaOS.

Values such as 1 Hz, 120 Hz, 144 Hz, or 260 Hz are examples only; actual behavior depends on each display, GPU, driver, mode, and connection.

## Logical scaling and coordinate foundation

The compositor now contains an initial, independently testable mathematical foundation:

- `output/scale.rs` represents scale factors as an exact numerator over 120, matching the Wayland fractional-scale convention. It provides checked logical/physical coordinate conversions and extent conversion.
- `output/coordinates.rs` separates global logical coordinates from output-local physical pixels. It defines half-open logical output bounds so adjacent monitor edges have deterministic ownership.
- The foundation includes unit tests for common scale factors, negative coordinates, overflow rejection, and mixed-scale output boundaries.

The rational scale is now also carried as an optional per-output setting in `DisplayOutput`. The output manager can configure it, automatic output placement uses the resulting logical width, pointer bounds use the effective scale, render targets retain the rational value, and Smithay receives a `Scale::Custom` value with a legacy integer advertisement.

Smithay's `wp_fractional_scale_manager_v1` global is now registered, and newly created fractional-scale objects receive a conservative 1.0 initial preference. Output-aware preferred-scale updates are not yet connected. This is **still not end-to-end fractional scaling**: surface-buffer viewport handling, cursor hotspots, clipping, hit-testing, and the actual Vulkan renderer must be migrated and tested together. The compositor must not advertise fractional scaling as a completed user-facing feature until those pieces are integrated.

The migration must preserve one invariant: global desktop positions and pointer locations are logical coordinates; each output converts them to its own physical pixel space exactly once. Surface buffers, cursor hotspots, clipping, hit-testing, and rendering must use the same transform and rounding policy.

## Current Status

**Initial DRM and output-management foundations implemented; Display Intelligence remains planned.**

The compositor currently provides:

- DRM device discovery and management.
- Connector and display mode discovery.
- Stable output identities.
- Output registration, removal, and reconciliation.
- Logical output layout.
- Wayland output publication.
- Initial DRM hotplug and session lifecycle integration.

The shared API contracts are present for display modes, capability profiles, VRR ranges, output layout and scale, automatic/manual policy, lifecycle events, and transactional configuration. A pure mode-selection policy is implemented in source, but it is not yet wired to a concrete hardware capability provider, KMS mode application, or application-facing IPC. Tests have not yet been executed in this work pass.\n\nThe following capabilities are not yet implemented as complete features:

- Physical display mode setting.
- Complete Vulkan frame rendering and physical presentation.
- VRR control.
- Runtime fixed-refresh configuration and KMS mode application.
- Adaptive refresh policy.
- Per-display automatic configuration.
- Settings → Display & Video interface.

The existing implementation is the foundation on which Display Intelligence will be built. It must not be confused with a completed display-management service.


## Native DRM timing retention (2026-11-27)

DRM discovery now retains the complete list of native connector timings as
pairs with their corresponding `DrmModeInfo` values on
`DrmOutputDescriptor`. The hardware-independent `DisplayOutput` registry
collapses duplicate logical modes rather than rejecting the entire output.
This prevents two timings that map to the same logical width, height and
reported millihertz from making an otherwise usable connector disappear.

`DrmOutputDescriptor::native_mode_for` currently resolves a logical mode to
the first retained native timing that maps to it. This is deterministic but
not a unique identity when multiple native timings collapse to one logical
mode. The current descriptor also retains the preferred native mode for
initial surface creation.

**Still incomplete:** native timing identity is not yet represented in
`OutputMode`; the output manager's mode changes are logical-only and are not
transactionally applied to an existing KMS surface. Do not claim live mode
switching, exact selection among colliding native timings, or rollback support
until that path is implemented and tested. Refresh-rate precision remains
limited by the current integer `vrefresh()` conversion.


## Native KMS mode staging (2026-11-27)

The native timing list is now copied into each `ManagedDrmSurface`. The
surface manager exposes `stage_mode(output_id, logical_mode)`, which:
- resolves the logical mode only when exactly one native DRM timing matches;
- rejects unsupported and ambiguous mappings without changing KMS state;
- stages the native timing with Smithay 0.7 `DrmSurface::use_mode()`;
- returns the previously active native mode for explicit restoration.

`restore_mode()` stages that previous timing again. Both operations are
staging operations: the new state becomes active only when the caller submits
a framebuffer through the existing presentation backend, which performs
atomic TEST_ONLY and then the KMS commit/modeset.

**Integration still required:** the display configuration transaction must
coordinate this API with allocation/rendering of a framebuffer at the target
size, call `restore_mode()` when rendering or presentation is abandoned, and
publish the new logical `OutputMode` only after the KMS commit completes.
The current output manager still updates logical state independently. No
build, test, or physical monitor validation has been run for this change.


## KMS mode rollback on presentation failure (2026-11-27)

Both DRM presentation paths now retain the previous native mode while a mode
change is staged. Vulkan presentation and CPU/GBM fallback restore that mode
when TEST_ONLY rejects the target framebuffer, when the KMS commit fails, or
when presentation setup/lifecycle validation fails before commit. A successful
presentation keeps the new mode and clears rollback protection on its VBlank;
the surface manager clears the unused alternate presenter's guard as well.

This is a KMS staging rollback, not yet a complete user-facing display
configuration transaction. If target-size rendering/allocation is abandoned
before either presenter is invoked, the owner must call restore_mode().
The compositor still needs to coordinate framebuffer allocation at the target
physical dimensions, submit the corresponding plane state, and update the
logical output registry only after successful KMS completion. No build, unit
test, or physical monitor validation has been run for these changes.


## Automatic KMS mode staging from output rendering (2026-11-27)

The server render dispatcher now reads the selected logical `OutputMode` for
each pending output and resolves it against the retained native timings before
constructing the presentation backend. A unique match is staged on the
Smithay DRM surface before rendering. The renderer's `RenderTarget` is derived
from the same logical output mode, and the Vulkan scanout path allocates or
reuses a framebuffer only when its physical width and height match that target.
The primary-plane state is then submitted through atomic TEST_ONLY and KMS
commit. CPU/GBM compatibility presentation reuses the same staged KMS mode.

The `ManagedDrmSurface::stage_native_mode` helper rejects unsupported or
ambiguous native mappings and arms both presenters with the previously active
mode. If the target mode is already active, it does not stage a redundant
modeset. Successful VBlank completion clears the unused presenter's rollback
guard; failed TEST_ONLY/commit restores the previous staged timing.

**Remaining transaction boundary:** `AureaCompositor::configure_output` still
updates the logical `OutputManager` and Wayland publication before the server
render dispatcher performs physical KMS staging. Therefore a physical modeset
failure can still leave the logical registry describing the requested mode
while the monitor remains in the previous mode. A server-owned configuration
request/acknowledgement path is still required to defer logical publication
until VBlank and restore the previous logical configuration on failure. This
integration has not been built or tested and has not been validated on physical
hardware.


## Abandoned render rollback (2026-11-27)

Each managed DRM surface now retains the previous native mode for the lifetime
of a staged change. If frame rendering fails before either presentation path
has a KMS flip pending, the server explicitly aborts the staged change and
restores the previous Smithay mode. It refuses this abort while a Vulkan or
CPU page flip remains pending, because KMS may still own the submitted frame.
A completed VBlank clears the retained previous mode and disarms the unused
presentation backend. This closes the pre-submission render-abandonment gap;
it does not yet defer the logical output registry/Wayland mode publication
until physical VBlank, which remains the outstanding transaction boundary.

## Two-phase output configuration

The compositor now separates validation from publication for output changes.
`OutputManager::staged_configuration` returns a fully validated candidate
without mutating the live registry or layout. The compositor exposes
`prepare_output_configuration` and `commit_prepared_output_configuration`:
the candidate can be retained while the DRM backend stages the native timing,
submits the matching framebuffer, and waits for VBlank. Only then should the
caller commit the candidate and publish the replacement Wayland output.

The existing `configure_output` convenience method still performs an
immediate logical-only change. It must not be used as the physical modeset
entry point. The staged API and its isolation test are in place, but the
end-to-end request queue from display settings through KMS completion to the
Wayland commit is not yet connected. Until that integration is complete,
physical mode changes must not be advertised as fully transactional.

## Overlapping KMS mode requests (2026-11-27)

The KMS surface manager now rejects a second, different mode request while an
earlier mode change is staged or has a page flip pending. This prevents a new
request from replacing the original rollback target before the first request
has either completed at VBlank or been explicitly aborted. Repeating the
already-staged target remains idempotent, and a request for the currently
active native mode remains a no-op.

This guard applies to both the public surface-manager staging API and the
server's automatic render-time staging path. It does not implement the missing
display-settings request queue or logical Wayland publication transaction.
