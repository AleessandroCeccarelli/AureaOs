<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Aurea Appearance and Aurea Lumen

## Purpose

AureaOS provides one system-wide appearance contract for the desktop shell, first-party applications and compatible third-party applications. The contract is implemented in `aurea-desktop-api`; canonical visual values and component guidance remain owned by AureaOS Human Interfaces.

This architecture has two related but independent responsibilities:

- **Aurea Appearance** resolves the visual presentation: Light/Dark/Automatic theme, semantic palette, geometry and accessibility preferences.
- **Aurea Lumen** adapts display color warmth according to user intent and available environmental context. It does not select the Light or Dark theme.

## Product naming

- **Aurea Appearance** — system appearance service and public API contract.
- **Aurea Lumen** — environmental display adaptation feature, replacing the generic user-facing Night Light name over time.
- **Aurea Palette** — the approved semantic color palette owned by Human Interfaces.
- **Aurea Form** — the approved geometry language: radii, surfaces, borders, spacing and proportions.
- **AureaOS Human Interfaces** — the overall design system and owner of visual specifications, tokens and reusable component rules.

These names describe product/API boundaries, not claims that all corresponding runtime services are already implemented.

## Aurea Appearance

### Theme policy

Applications may request or observe the system theme preference:

- **Automatic** (API compatibility value: `System`) follows the resolved system appearance policy.
- **Light** selects the light color scheme.
- **Dark** selects the dark color scheme.

The resolved `ColorScheme` is always explicit. A preference is not the same as the currently resolved scheme.

### Shared palette and geometry

The appearance snapshot identifies the canonical `PaletteId::Aurea` and `FormProfile::Aurea`. Applications consume semantic roles and token names rather than defining competing system palettes or arbitrary system-wide geometry.

Color values and geometry values remain in the single canonical source:

`aurea-human-interfaces/design-system/tokens/index.css`

The legacy theme wrappers must not define alternate values. Current semantic color roles and geometry roles map to canonical token names.

The API exposes semantic identity, not raw CSS parsing or a second token database. Native UI components and application adapters resolve these identifiers through the appropriate Human Interfaces implementation.

### Accessibility and motion

The appearance contract continues to expose reduced motion, reduced transparency and high-contrast preferences. Consumers must preserve readable contrast and interaction hierarchy in both themes. Reduced-transparency behavior is defined centrally in the canonical token source.

## Aurea Lumen

Aurea Lumen applies a system-level warm color adjustment through the compositor output path. It is separate from application content filters and separate from Light/Dark theme selection.

### Operating modes

- **Manual** — the user explicitly controls activation.
- **Schedule** — follows a local-clock interval configured by the user.
- **Solar** — follows sunrise and sunset using a permission-aware location/time service. A location permission must not be silently assumed.
- **Ambient** — follows a supported ambient-light sensor.
- **Automatic** — prefers reliable ambient sensing and falls back to an available solar or local schedule. If no trusted source is available, it must remain predictable and expose its effective state.

The local schedule works without location access and supports intervals crossing midnight. Invalid and empty intervals are rejected.

Ambient thresholds use a hysteresis band to prevent rapid switching when measured light fluctuates near a boundary. Sensor absence, stale readings or errors must not be interpreted as darkness.

### User controls

The user can enable/disable Lumen, choose a mode, adjust warmth strength, configure a local schedule and configure ambient thresholds. Manual override must remain available even when automatic sources exist.

### Ownership and implementation boundary

- Desktop API defines portable preferences and policy data types.
- Shell/settings owns user interaction and preference persistence.
- Sensor and time providers own ambient measurements, timezone and solar calculations.
- The display policy coordinator chooses the effective Lumen state.
- Compositor owns application of the final display color adjustment.
- Applications may observe the resolved system appearance; they do not independently apply Lumen to their content.

The current shell Night Light model and Vulkan output adjustment path are existing implementation foundations. The new API contract does not imply that ambient sensors, solar scheduling, preference persistence, Settings UI, or application-facing IPC are complete.

## Application developer contract

Compatible applications should:

1. Observe the shared appearance provider rather than guessing theme from environment variables.
2. Use semantic color roles and the canonical Aurea palette.
3. Use Aurea Form geometry tokens and shared Human Interfaces components.
4. React to appearance/accessibility updates without restarting.
5. Keep app-specific accents local only when explicitly permitted by product/design rules; never redefine the system palette.
6. Avoid applying a second night-color filter to application content.

## Canonical sources

- API: `components/aurea-desktop-api/src/appearance.rs`
- Lumen API: `components/aurea-desktop-api/src/appearance/lumen.rs`
- Color tokens: `aurea-human-interfaces/design-system/tokens/index.css`
- Design system: `aurea-human-interfaces/design-system/README.md`
- Color system: `docs/human-interfaces/COLOR-SYSTEM.md`

## Status

The shared Rust contracts for theme identity, palette/form identity and Lumen preferences are defined. The existing appearance provider trait remains the integration point for runtime snapshots and subscriptions.

Concrete appearance provider, environmental sensor discovery, solar/time provider, Lumen policy coordinator, persistence, native settings controls and application-facing IPC remain implementation work. Do not describe these planned services as complete until code and tests verify them.
