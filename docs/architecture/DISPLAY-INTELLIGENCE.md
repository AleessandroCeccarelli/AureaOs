
# Aurea Display Intelligence

## Purpose

Aurea Display Intelligence is the planned AureaOS subsystem for discovering display capabilities and selecting an appropriate display mode and refresh strategy.

Its goal is to make display configuration automatic by default while keeping clear, predictable manual controls available to the user.

This is an architectural direction, not an implemented service. The current repository does not yet contain this subsystem or a complete DRM/KMS display pipeline.

## Product principles

- AureaOS, not the Debian base distribution, owns display policy and user-facing decisions.
- Linux kernel drivers and DRM/KMS remain the hardware interface used to discover capabilities and apply supported configurations.
- The monitor and graphics driver define the actual limits. AureaOS must never request unsupported resolutions, refresh rates, or VRR ranges.
- Automatic management is the default experience; manual configuration remains available.
- Each connected display is discovered and managed independently.
- Capability discovery and policy decisions are separate from the compositor's rendering and protocol responsibilities.

## Subsystem architecture

### Display Capability Service

Discovers and maintains a capability profile for each connected display, using the information exposed by DRM/KMS, connector properties, EDID and the active graphics driver.

The profile should include, when available:

- Stable display and connector identity.
- Supported resolutions and display modes.
- Available fixed refresh rates for each mode.
- Whether the connector/CRTC path supports variable refresh rate.
- The effective VRR range and relevant driver constraints, when they can be established reliably.
- Physical and logical display properties needed by the desktop.

EDID data is useful input, but it is not by itself a guarantee that a feature is usable. Driver-reported properties and the active display pipeline must also be checked. Unknown or contradictory capabilities must be treated conservatively.

### Aurea VRR

The adaptive-refresh strategy for displays and display pipelines that support VRR.

It should coordinate presentation with the actual frame cadence and the effective VRR limits. It may allow the display to follow low or high frame rates within the supported range, subject to driver behavior, synchronization, and panel limitations.

The service must not assume that every VRR display supports 1 Hz. Minimum and maximum rates are hardware- and driver-dependent. Low Framerate Compensation (LFC), when available, may help handle frame rates below the native VRR minimum; it must not be assumed without capability evidence.

### Aurea Fixed Refresh

The fallback strategy for displays that do not support VRR, or for situations where VRR is unavailable or unsuitable.

It selects among the fixed modes exposed by the display and graphics driver. Selection should consider the active workload, presentation cadence, user preferences, power use, and stability.

It must not claim to provide continuous variable refresh on fixed-refresh hardware.

### Display Policy Engine

Coordinates capability profiles, user preferences, desktop context, and system constraints to choose a display policy.

Context may include compositor presentation timing, frame cadence, fullscreen state, known application presentation hints, desktop animation activity, and power policy.

The initial design should use explicit system/compositor signals rather than inspecting application pixels or private content.

The policy engine should:

- Choose a valid resolution and refresh strategy for each display.
- Prefer VRR when supported and appropriate.
- Otherwise select a suitable fixed refresh mode.
- Avoid unnecessary mode changes and oscillation through hysteresis, minimum dwell times, and safe transitions.
- Respect manual user choices and explicit application/system constraints.
- Handle display hotplug, session changes, suspend/resume, and driver capability changes.
- Provide a safe fallback if capability discovery or mode application fails.

### DRM/KMS Backend

Applies the decisions through the kernel's DRM/KMS interfaces and the active graphics driver. It reports success, failure, and the effective state back to the subsystem.

The backend is an implementation boundary, not the policy owner. AureaOS decides what it wants; DRM/KMS and the driver determine whether the requested configuration can be applied.

### Compositor integration

The Wayland compositor supplies trusted presentation and output lifecycle information and applies the display state changes requested by the display subsystem.

It should not contain the entire adaptive policy engine.

The boundary between the service, compositor, and DRM/KMS backend must be defined before implementation. The design must avoid competing owners of the same DRM device or display state.

## User experience

The planned **Settings → Display & Video** interface should provide:

- **Automatic — AureaOS:** recommended default. AureaOS chooses a suitable resolution and refresh strategy using detected capabilities and current context.
- **Manual:** lets the user choose a supported resolution and fixed refresh rate where available.
- A clear indication of the active display, supported modes, VRR availability, and current management mode.
- A safe confirmation/revert flow when applying a manual display configuration, so an unusable mode does not leave the user without a working display.
- Per-display settings for multi-monitor systems.

The UI must distinguish a selected fixed mode from a live VRR operating frequency. With VRR, the selected display mode defines the usable mode and VRR envelope; the instantaneous refresh can vary during presentation.

## Operating rules

1. Discover capabilities before offering choices.
2. Treat the effective driver and display pipeline as authoritative for what can actually be applied.
3. Never infer a universal refresh range from marketing labels or EDID alone.
4. Keep resolution selection and refresh policy related but distinct: not every resolution supports every refresh rate or VRR range.
5. Prefer stable transitions over frequent mode switching.
6. Make automatic behavior explainable and allow the user to override it.
7. Fall back to a known working fixed mode if VRR or adaptive policy cannot be used.
8. Re-evaluate capabilities after hotplug, resume, GPU/session changes, and relevant driver events.

## Multi-monitor behavior

Each display has its own capability profile and policy state.

AureaOS should be able to use VRR on one display and fixed refresh on another. Shared GPU, CRTC, bandwidth, synchronization, and compositor constraints must be considered before applying simultaneous configurations.

## Implementation direction

The work should be delivered incrementally:

1. Define display identity and capability data models.
2. Complete DRM connector discovery and stable output identity.
3. Implement a DRM/KMS mode-application boundary with explicit ownership and error reporting.
4. Expose effective display capabilities and state to the compositor and system service.
5. Implement fixed-refresh selection first as a reliable fallback.
6. Add VRR capability detection and controlled enablement where the complete pipeline supports it.
7. Implement context-aware policy, safeguards, and per-display settings.
8. Validate behavior across different drivers, monitors, hotplug, suspend/resume, and multi-monitor configurations.

No specific minimum or maximum refresh rate is guaranteed by AureaOS. Values such as 1 Hz, 120 Hz, 144 Hz, or 260 Hz are examples only; actual behavior depends on each display, GPU, driver, mode, and connection.

## Current status

**Planned architecture — not implemented.** The existing compositor contains early output models and DRM discovery work, but it does not yet provide complete mode setting, VRR control, adaptive refresh policy, or the Settings interface described here.
