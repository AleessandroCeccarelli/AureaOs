
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Aurea Display Intelligence

## Purpose

Aurea Display Intelligence is the planned AureaOS subsystem for discovering display capabilities and selecting appropriate display modes and refresh strategies.

Its goal is to make display configuration automatic by default while keeping clear, predictable manual controls available to the user.

The compositor already provides initial DRM device discovery, connector discovery, output lifecycle management, and Wayland output publication.

However, Aurea Display Intelligence itself is not yet implemented. Complete DRM/KMS mode setting, physical frame presentation, VRR control, and adaptive refresh policy remain future work.

## Product principles

- AureaOS, not the Debian base distribution, owns display policy and user-facing decisions.
- Linux kernel drivers and DRM/KMS provide the hardware interfaces used to discover capabilities and apply supported configurations.
- The monitor and graphics driver define the actual limits. AureaOS must never request unsupported resolutions, refresh rates, or VRR ranges.
- Automatic management is the default experience; manual configuration remains available.
- Each connected display is discovered and managed independently.
- Capability discovery and policy decisions are separate from the compositor's rendering and protocol responsibilities.
- Existing compositor responsibilities must not be duplicated by a separate display service.

## Subsystem architecture

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

### Compositor integration

The Wayland compositor supplies trusted presentation and output lifecycle information and applies display state changes requested by the display subsystem.

The current compositor already provides initial output lifecycle management and publishes registered outputs as Wayland output globals.

It does not yet provide complete physical display mode setting or frame presentation.

The compositor should not contain the entire adaptive policy engine.

The boundary between the display subsystem, compositor, and DRM/KMS backend must be explicit. The design must avoid competing owners of the same DRM device or display state.

## User experience

The planned **Settings → Display & Video** interface should provide:

- **Automatic — AureaOS:** recommended default. AureaOS chooses a suitable resolution and refresh strategy using detected capabilities and current context.
- **Manual:** lets the user choose a supported resolution and fixed refresh rate where available.
- A clear indication of the active display, supported modes, VRR availability, and current management mode.
- A safe confirmation and revert flow when applying a manual display configuration, so an unusable mode does not leave the user without a working display.
- Per-display settings for multi-monitor systems.

The UI must distinguish a selected fixed mode from a live VRR operating frequency.

With VRR, the selected display mode defines the usable mode and VRR envelope; the instantaneous refresh can vary during presentation.

This interface is a product requirement, not an implemented feature.

## Operating rules

1. Discover capabilities before offering choices.
2. Treat the effective driver and display pipeline as authoritative for what can actually be applied.
3. Never infer a universal refresh range from marketing labels or EDID alone.
4. Keep resolution selection and refresh policy related but distinct: not every resolution supports every refresh rate or VRR range.
5. Prefer stable transitions over frequent mode switching.
6. Make automatic behavior explainable and allow the user to override it.
7. Fall back to a known working fixed mode if VRR or adaptive policy cannot be used.
8. Re-evaluate capabilities after hotplug, resume, GPU/session changes, and relevant driver events.
9. Never apply a display configuration without a validated path to restore a working state.

## Multi-monitor behavior

Each display has its own capability profile and policy state.

AureaOS should be able to use VRR on one display and fixed refresh on another.

Shared GPU, CRTC, bandwidth, synchronization, and compositor constraints must be considered before applying simultaneous configurations.

The implementation must also account for the possibility that different outputs have different capabilities and limitations.

## Implementation direction

The work should be delivered incrementally:

1. **Display identity and output foundations — initial implementation available.** The compositor already provides stable output identities, connector discovery, output registration, removal, and logical layout management.
2. **DRM device and output lifecycle — initial implementation available.** Device discovery, lifecycle integration, output reconciliation, and Wayland output publication are present in the compositor.
3. **Physical display pipeline — incomplete.** Implement a concrete rendering and presentation path and a DRM/KMS mode-application boundary with explicit ownership and error reporting.
4. **Capability reporting — incomplete.** Extend discovered output information into reliable per-display capability profiles and expose effective state through defined interfaces.
5. **Fixed-refresh configuration — planned.** Implement and validate fixed-mode selection as the first reliable configuration strategy.
6. **VRR capability detection — planned.** Detect effective support across the complete display pipeline.
7. **Controlled VRR enablement — planned.** Add VRR only where the complete pipeline supports it and safe transitions can be guaranteed.
8. **Display policy — planned.** Implement context-aware policy, safeguards, and per-display settings.
9. **Validation — required.** Test different drivers, monitors, hotplug, suspend/resume, and multi-monitor configurations.

No specific minimum or maximum refresh rate is guaranteed by AureaOS.

Values such as 1 Hz, 120 Hz, 144 Hz, or 260 Hz are examples only; actual behavior depends on each display, GPU, driver, mode, and connection.

## Current status

**Initial DRM and output-management foundations implemented; Display Intelligence remains planned.**

The compositor currently provides:

- DRM device discovery and management.
- Connector and display mode discovery.
- Stable output identities.
- Output registration, removal, and reconciliation.
- Logical output layout.
- Wayland output publication.
- Initial DRM hotplug and session lifecycle integration.

The following capabilities are not yet implemented as complete features:

- Physical display mode setting.
- Complete frame rendering and presentation.
- VRR control.
- Fixed-refresh selection policy.
- Adaptive refresh policy.
- Per-display automatic configuration.
- Settings → Display & Video interface.

The existing implementation is the foundation on which Display Intelligence will be built. It must not be confused with a completed display-management service.
