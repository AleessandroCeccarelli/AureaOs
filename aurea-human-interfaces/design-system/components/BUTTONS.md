<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Button Specification

**Status:** Design specification

## Purpose
Buttons trigger actions. Their appearance communicates priority consistently across applications.

## Variants
- **Primary:** the main action; use sparingly, normally one dominant primary action per focused workflow.
- **Secondary:** supporting actions.
- **Subtle:** low-emphasis actions in toolbars and compact surfaces.
- **Destructive:** deletion, removal or irreversible actions.
- **Icon button:** compact action with an accessible name and tooltip when meaning is not obvious.

## States
Define default, hover, focus-visible, pressed, disabled and loading. Use selected/toggled only for controls representing persistent state. Loading prevents duplicate submission and communicates progress.

## Behavior
Enter and Space activate a focused button. Focus remains visible. Significant destructive actions require confirmation. Disabled controls cannot activate.

## Appearance and accessibility
Use shared tokens for height, padding, radius, icon size, typography and focus. Verify both themes and text scaling. Never communicate state by color alone.

## Avoid
Multiple competing primary actions, unlabeled icon buttons, inconsistent geometry and animation that delays activation.
