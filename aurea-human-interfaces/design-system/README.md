<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Design System

This directory is the canonical source for the shared visual language used by the AureaOS desktop and first-party applications.

## Structure

- `tokens/` — semantic color, surface, typography and elevation values. `tokens/index.css` is the canonical token entry point.
- `components/` — component specifications, states, behavior and accessibility requirements.
- `patterns/` — recurring task flows and compositions built from shared components.

## Authority

The design system owns shared visual decisions. Applications must not invent parallel palettes, spacing scales, menu conventions or control styles.

The `themes/` directory at the Human Interfaces root is retained only for compatibility. It must not become a second source of truth.

## Component status

Specifications describe the intended contract; they do not imply that native Rust components or system services already exist. Implementation status is tracked in the relevant runtime crates and architecture documentation.

## Rules

- Use semantic tokens rather than hard-coded colors.
- Review every component in light and dark themes.
- Include keyboard, focus, disabled and error behavior.
- Respect reduced motion and accessibility preferences.
- Keep files focused: one component family or pattern per document.
- Update the relevant index whenever a new specification is added.
