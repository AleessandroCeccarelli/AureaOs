<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Multi-Monitor Policy

## Decision

AureaOS supports **up to four active display outputs at the same time**.

This is a product and maintainability decision, not a universal Linux, DRM, GPU, or driver limit. The value is centralized in `MAX_ACTIVE_OUTPUTS` so it can be reviewed and changed deliberately.

A display may be discovered and remain registered while disabled. The four-output limit applies to enabled outputs, not to the number of devices the system can identify.

## Why four displays

Four active displays are a practical balance between usability and implementation complexity.

For usability, this supports common and advanced desktop arrangements: a primary display, one or two additional work areas, and another display for references, communication, monitoring, or creative tools. It covers a broad range of multi-monitor workflows without making very large display walls a baseline product requirement.

For maintainability, every additional output increases the combinations that must be handled and tested:

- Display modes, resolutions, refresh rates, and scale factors.
- Logical layouts, including displays positioned above, below, left, or right of one another.
- Pointer hit-testing and movement across boundaries and gaps.
- Window placement, focus, and output association.
- Hotplug, unplug, reconfiguration, and recovery.
- DRM/KMS resources such as connectors, CRTCs, planes, bandwidth, and synchronization.
- Rendering, presentation, and failure handling across multiple outputs.
- Settings, diagnostics, automated tests, and hardware validation.

Four is therefore a deliberate initial ceiling: useful for real desktop work while keeping the number of combinations manageable for a young compositor.

## Operational rules

1. No more than four outputs may be enabled simultaneously.
2. A fifth connected output may be discovered and registered as disabled; it must not be silently discarded.
3. Enabling a fifth output must fail with a clear error and leave the current configuration unchanged.
4. Batch registration and hotplug reconciliation must validate the final active set before mutating state.
5. Disabling or unplugging an output releases its active slot.
6. Disabled outputs do not consume one of the four active slots.
7. Output layout is not assumed to be a single horizontal row. The layout model must support different relative positions and non-aligned display edges.

The compositor must preserve existing working outputs if a new device cannot be enabled within the limit.

## Hardware constraints

The policy ceiling does not guarantee that every machine can drive four displays. The actual number of usable outputs, modes, resolutions, and refresh rates depends on the GPU, driver, physical connectors, available CRTCs and planes, memory bandwidth, and synchronization constraints.

A system may support fewer than four displays, or may support four only at selected modes. AureaOS must rely on discovered and validated capabilities rather than promise a particular hardware configuration.

## Future review

The limit is intentionally centralized and should only be raised after evidence from implementation, automated tests, and representative hardware validation demonstrates that the added complexity is justified.

Until that review, four active displays remain the supported AureaOS design target.
