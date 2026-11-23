<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Button Specification

**Status:** Design specification

## Purpose

Buttons trigger actions. Their appearance communicates priority consistently across AureaOS applications.

## Variants

- **Primary:** main action; use sparingly, normally one dominant primary action per focused workflow. Use the canonical blue tonal gradient and high-contrast label.
- **Secondary:** supporting action on a quiet elevated surface, using shared control fill and border tokens.
- **Subtle:** low-emphasis action in toolbars and compact surfaces; no permanent heavy fill.
- **Destructive:** deletion, removal or irreversible actions. Use semantic critical color for label or emphasis, not a permanently alarming full-surface treatment for routine actions.
- **Icon button:** compact action with an accessible name and tooltip when meaning is not obvious.

## Visual treatment

Use the shared control radius, typography, spacing, border and focus tokens. Primary actions use the shared primary gradient; secondary controls use the neutral light/dark control surface. Keep highlights and shadows restrained. Do not add per-app gradients, glows or bespoke button geometry.

## States

Define default, hover, focus-visible, pressed, disabled and loading. Use selected/toggled only for controls representing persistent state.

- Hover: slightly brighter tonal surface or accent, with a subtle border change.
- Focus-visible: clear shared focus ring; never rely on hover styling.
- Pressed: slightly deeper tonal surface; no large movement or delayed response.
- Disabled: reduced emphasis and disabled semantics; do not communicate disabled state by opacity alone.
- Loading: prevent duplicate submission and communicate progress without changing button geometry.

## Behavior and accessibility

Enter and Space activate a focused button. Focus remains visible. Significant destructive actions require confirmation. Disabled controls cannot activate. Text and icon contrast must remain sufficient in both themes and at supported text scales.

## Avoid

Multiple competing primary actions, unlabeled icon buttons, inconsistent geometry, excessive glow, glossy bevels that reduce legibility and animation that delays activation.
