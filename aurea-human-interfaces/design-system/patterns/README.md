<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Patterns

Canonical specifications for recurring interface compositions and interaction patterns across the AureaOS desktop and first-party applications.

## Pattern index

- [Desktop shell](DESKTOP-SHELL.md) — dock, launcher, workspaces and system surfaces.
- [Application navigation](APPLICATION-NAVIGATION.md) — stable in-app navigation and hierarchy.
- [Feedback and notifications](FEEDBACK-AND-NOTIFICATIONS.md) — inline feedback, toasts, system notifications and glass material.
- [Forms and settings](FORMS-AND-SETTINGS.md) — grouping, saving behavior and setting controls.

## Shared visual direction

Patterns use the AureaOS semantic tokens and component contracts. The visual language is Windows 11-inspired: clean layered surfaces, blue/cyan accents, restrained gradients, rounded geometry and subtle depth. The dock and floating system surfaces may use more pronounced translucency; application content remains calm and readable.

Notifications are a signature glass surface: cool translucent material, real backdrop blur when supported, subtle edge highlight and soft shadow. This effect is intentionally concentrated on shell-owned floating surfaces rather than applied indiscriminately to every window.

## Principles

- Familiarity first; refinement through integration.
- Shared commands, surfaces and interaction states remain consistent.
- Use transparency only when it clarifies system-level layering.
- Preserve contrast, focus and legibility in both themes.
- Respect reduced motion and reduced transparency.
- A design pattern is a specification, not proof of runtime implementation.
