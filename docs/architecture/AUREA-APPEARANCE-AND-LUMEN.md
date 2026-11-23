<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Aurea Appearance and Aurea Lumen — Exploratory Architecture

## Project maturity and intent

AureaOS is in an early, embryonic development phase. This document records the current product direction and API experiments; it is not a final visual standard, a release promise, or a guarantee that every described capability will ship.

Names, modes, architecture, APIs, palette values, geometry, interaction patterns and implementation choices may change during development. They should be revised when testing, accessibility review, hardware validation, developer feedback or better technical options justify it.

The terms **proposed**, **experimental**, **implemented contract**, and **runtime implemented** must be used accurately. A type or document does not prove that a complete service exists.

## Current direction

The current naming proposal is:

- **Aurea Appearance** — system appearance and accessibility preferences.
- **Aurea Lumen** — environmental display color adaptation, distinct from Light/Dark theme selection.
- **Aurea Palette** — the current proposed name for the semantic color system.
- **Aurea Form** — the current proposed name for geometry and surface language.
- **AureaOS Human Interfaces** — the evolving design-system project.

These names are working product terminology and may be renamed or reshaped.

## Aurea Appearance — proposed contract

The current API direction covers Automatic/System, Light and Dark theme preferences, a resolved color scheme, semantic palette identity, form profile and accessibility preferences.

The current `PaletteId::Aurea` and `FormProfile::Aurea` are provisional identifiers, not a freeze of colors or shapes. Applications should consume semantic roles and shared tokens rather than assume that present-day hex values or corner radii are permanent.

The current token file is a working source of truth for the present design iteration. It may evolve or be reorganized as Human Interfaces matures.

## Aurea Lumen — proposed feature

Aurea Lumen is the current name for system-level warm display adaptation. Its proposed modes are:

- **Manual** — explicit user control.
- **Schedule** — a local-clock interval.
- **Solar** — sunrise/sunset using a permission-aware location/time source.
- **Ambient** — a supported ambient-light sensor.
- **Automatic** — select an available trusted source according to policy.

This list and its fallback order are proposals subject to feasibility, privacy, usability and hardware validation. Sensor absence, stale readings or errors must not be silently interpreted as darkness. Local scheduling should remain possible without location access. Ambient hysteresis is a candidate technique to avoid rapid switching.

Lumen is intended to remain conceptually separate from the Light/Dark theme, although the final UX and implementation may evolve.

## API and implementation boundaries — current proposal

- Desktop API defines portable contracts and preferences.
- Shell/settings owns user interaction and, when implemented, preference persistence.
- Sensor and time providers supply measurements and time/solar data.
- A policy coordinator determines effective behavior.
- Compositor owns physical output color adjustment.
- Compatible applications may observe system appearance; they should not independently apply a duplicate system display filter.

These are proposed ownership boundaries, not a statement that all services or IPC are implemented.

## Guidance for application developers

During this early phase, developers should:

1. Prefer the shared appearance contracts and semantic tokens over hard-coded assumptions.
2. Expect names, token values and API details to evolve before a stable compatibility commitment.
3. Avoid depending on provisional numeric colors or geometry values.
4. Follow accessibility and readable-contrast requirements.
5. Treat undocumented behavior as unstable and check the repository for updates.

A future stability policy and API versioning/deprecation process must be defined before promising long-term compatibility.

## Current implementation status

Rust contracts currently exist for appearance identity and Lumen preferences. The existing shell Night Light model and Vulkan output adjustment path are foundations, not a complete Lumen service.

Concrete appearance provider, sensor discovery, solar/time provider, policy coordinator, persistence, native settings controls and application-facing IPC remain implementation work. Physical hardware validation is also outstanding.

## Review and evolution

Revisit this document as implementation and testing provide evidence. Do not preserve a decision merely because it has been documented. Record significant changes and their rationale in the development log, update the API and Human Interfaces specifications together, and distinguish breaking experimental changes from stable contracts.
