<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Feedback and Notifications Pattern

**Status:** Design specification
**Visual reference:** Windows 11 notification refinement with original AureaOS glass materials.

## Purpose

Feedback confirms an action. Notifications communicate events that may matter beyond the current focused task.

## Choose the right surface

- **Inline feedback:** validation or status directly related to current content.
- **Toast:** brief, non-blocking confirmation of a routine action.
- **Notification:** an event that may need attention later or originates outside the current task.
- **Dialog:** only for a decision that must be resolved before continuing.

Do not use a notification for routine field validation or a modal dialog for a passive status update.

## Aurea notification glass

Notifications are a signature floating system surface. Their appearance must be consistent across all applications because the shell owns their presentation.

- Use the canonical notification fill, border, highlight, radius, shadow and backdrop tokens from `design-system/tokens/index.css`.
- In light mode, use a cool, near-white translucent glass with a delicate white edge highlight and soft neutral shadow.
- In dark mode, use a deep blue translucent glass with a subtle cool edge highlight; keep text bright and legible.
- When supported, apply backdrop blur and restrained saturation to the notification surface only. Never blur the notification text, icon or controls.
- Use a fine top/edge highlight to suggest glass thickness; avoid neon outlines, strong glow or thick glossy bevels.
- Keep notification content visually calm: source identity, concise title, message and optional actions have a stable hierarchy.
- Group related notifications with consistent spacing and avoid stacking that obscures the desktop or dock.
- Notification cards use consistent rounded geometry and padding. Actions use the shared button specification.

The glass effect must be real backdrop material supplied by the rendering surface/compositor, not a blurred screenshot baked into the card. If backdrop blur is unavailable, use the tokenized opaque/elevated fallback. Under reduced-transparency preference, remove blur and translucent highlights while preserving readable surface contrast and hierarchy.

## Behavior

Messages state what happened and, when useful, what the user can do next. Notifications identify their source and expose only supported actions. Dismissal, grouping, expiry and priority follow system policy. Critical alerts must not be disguised as ordinary decorative cards.

The shell owns notification presentation and stacking. Applications submit notification content through the system notification service; they do not draw their own fake system notification surfaces.

## Accessibility

Important status changes are announced accessibly. Severity is not communicated by color alone. Avoid repeatedly announcing transient updates. Support keyboard dismissal/action access, readable text scaling, reduced motion and reduced transparency.
