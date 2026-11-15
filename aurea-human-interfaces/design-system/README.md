<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Design System

This directory is the canonical source for shared visual and interaction rules used by the desktop and first-party applications.

## Structure

- `tokens/` — semantic colors, surfaces, typography and elevation. `tokens/index.css` is the only token source.
- `components/` — reusable component specifications, states, behavior and accessibility.
- `patterns/` — recurring task flows and compositions built from shared components.
- `KEYBOARD-AND-COMMAND-CONVENTIONS.md` — shared shortcut defaults, command placement and ownership.

## Product direction

AureaOS follows familiarity first. Windows is the primary reference for desktop conventions and common workflows; KDE Plasma informs the floating bottom dock and desktop flexibility; macOS informs consistency, visual finish and notification refinement.

These references guide usability, not copying. Keep familiar commands, locations and shortcuts predictable while using original AureaOS visual assets.

## Authority

The design system owns shared visual and interaction decisions. Applications must not invent parallel palettes, spacing scales, menu conventions or shortcut systems.

The root `themes/` directory and theme-specific token files are compatibility entry points only. `tokens/index.css` remains authoritative.

## Status and rules

Specifications define intended contracts; they do not imply that native Rust components or system services already exist.

- Use semantic tokens rather than hard-coded colors.
- Review every component in light and dark themes.
- Include keyboard, focus, disabled and error behavior.
- Respect reduced motion, reduced transparency and accessibility preferences.
- Keep files focused: one component family or pattern per document.
- Update the relevant index whenever a specification is added.
